use cw20::{Cw20Coin, Logo, MinterResponse};
use cw20_base::msg::{InstantiateMarketingInfo, InstantiateMsg as Cw20InstantiateMsg};
use cw_storage_plus::{Item, Map};

use cosmwasm_std::{to_json_binary, Addr, DepsMut, Reply, Response, StdError, SubMsg, Uint128, WasmMsg};
use cw_utils::{parse_reply_instantiate_data, MsgInstantiateContractResponse};

use crate::{
    curve::Curve,
    error::ContractError,
    execute::Context,
    math::mul_u256,
    models::{
        account::{AccountStats, SwapStats},
        ohlc::OhlcBar,
    },
    msg::InstantiateMsg,
    token::Token,
};

pub const CW20_INSTANTIATE_REPLY_ID: u64 = 1;

// Curve consists of reserve amounts & other vars used by CP AMM
pub const CURVE: Item<Curve> = Item::new("curve");

// Initial "virtual" quote reserve amount
pub const QUOTE_RESERVE_VIRTUAL: Item<Uint128> = Item::new("virtual_quote_reserve");

// Token addresses/denoms for base and quote tokens
pub const QUOTE_TOKEN: Item<Token> = Item::new("q_token");
pub const BASE_TOKEN: Item<Token> = Item::new("b_token");

// If defined, the operator is the only party authorized to initiate swaps on
// behalf of users. This is for the case when the curve is controlled by some
// other smart contract.
pub const OPERATOR_ADDR: Item<Addr> = Item::new("operator_addr");

// Fee configuration
pub const FEE_ADDR: Item<Addr> = Item::new("fee_addr");
pub const FEE_PCT_BUY: Item<Uint128> = Item::new("b_fee");
pub const FEE_PCT_SELL: Item<Uint128> = Item::new("s_fee");

// Historical price OHLC time series
pub const OHLC_BARS: Map<u64, OhlcBar> = Map::new("ohlc_bars");

// Account-level statistics
pub const ACCOUNT_STATS: Map<&Addr, AccountStats> = Map::new("account_stats");

// Global Statistic
pub const NET_TAKER_FEE: Item<Uint128> = Item::new("net_taker_fee");
pub const NET_MAKER_FEE: Item<Uint128> = Item::new("net_maker_fee");
pub const TAKER_STATS: Item<SwapStats> = Item::new("taker_stats");
pub const MAKER_STATS: Item<SwapStats> = Item::new("maker_stats");

/// Top-level initialization of contract state
pub fn init(
    ctx: Context,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let Context { deps, env, .. } = ctx;
    let InstantiateMsg {
        operator_addr,
        base_token,
        base_reserve,
        quote_reserve,
        quote_token,
        taker_fee_pct,
        maker_fee_pct,
        fee_addr,
    } = msg;

    QUOTE_TOKEN.save(deps.storage, &quote_token.token)?;
    FEE_ADDR.save(deps.storage, &deps.api.addr_validate(fee_addr.as_str())?)?;
    FEE_PCT_BUY.save(deps.storage, &taker_fee_pct.min(1_000_000u128.into()))?;
    FEE_PCT_SELL.save(deps.storage, &maker_fee_pct.min(1_000_000u128.into()))?;
    NET_TAKER_FEE.save(deps.storage, &Uint128::zero())?;
    NET_MAKER_FEE.save(deps.storage, &Uint128::zero())?;
    QUOTE_RESERVE_VIRTUAL.save(deps.storage, &quote_reserve)?;
    TAKER_STATS.save(deps.storage, &SwapStats::default())?;
    MAKER_STATS.save(deps.storage, &SwapStats::default())?;

    if let Some(operator_addr) = operator_addr {
        OPERATOR_ADDR.save(deps.storage, &deps.api.addr_validate(operator_addr.as_str())?)?;
    }

    CURVE.save(
        deps.storage,
        &Curve {
            k: mul_u256(base_reserve, quote_reserve)?,
            base_decimals: base_token.decimals,
            quote_decimals: quote_token.decimals,
            base_reserve,
            quote_reserve,
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_submessage(SubMsg::reply_on_success(
            WasmMsg::Instantiate {
                admin: Some(env.contract.address.to_string()),
                code_id: base_token.code_id.into(),
                msg: to_json_binary(&Cw20InstantiateMsg {
                    decimals: base_token.decimals,
                    name: base_token.name,
                    symbol: base_token.symbol.to_owned(),
                    marketing: Some(InstantiateMarketingInfo {
                        description: base_token.description,
                        logo: base_token.image_url.and_then(|url| Some(Logo::Url(url.to_owned()))),
                        project: base_token.project_url,
                        marketing: Some(env.contract.address.to_string()),
                    }),
                    initial_balances: vec![Cw20Coin {
                        address: env.contract.address.to_string(),
                        amount: base_reserve,
                    }],
                    mint: Some(MinterResponse {
                        minter: env.contract.address.to_string(),
                        cap: base_token.cap,
                    }),
                })?,
                funds: vec![],
                label: format!("{} CW20 Pro Token", base_token.symbol),
            },
            CW20_INSTANTIATE_REPLY_ID,
        )))
}

pub fn handle_cw20_instantiate_reply(
    deps: DepsMut,
    reply: Reply,
) -> Result<(), ContractError> {
    let MsgInstantiateContractResponse { contract_address, .. } = parse_reply_instantiate_data(reply.to_owned())
        .map_err(|e| ContractError::Std(StdError::GenericErr { msg: e.to_string() }))?;

    BASE_TOKEN.save(deps.storage, &Token::Address(Addr::unchecked(contract_address)))?;

    Ok(())
}
