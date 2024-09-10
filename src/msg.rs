use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128, Uint256, Uint64};
use cw20::Cw20ReceiveMsg;

use crate::{
    models::{
        account::{AccountStats, SwapStats},
        config::Config,
    },
    token::Token,
};

#[cw_serde]
pub struct BaseTokenInitArgs {
    pub code_id: Uint64,
    pub symbol: String,
    pub decimals: u8,
    pub name: String,
    pub image_url: Option<String>,
    pub description: Option<String>,
    pub project_url: Option<String>,
    pub cap: Option<Uint128>,
}

#[cw_serde]
pub struct QuoteTokenInitArgs {
    pub token: Token,
    pub decimals: u8,
}

#[cw_serde]
pub struct InstantiateMsg {
    /// Addr of other smart contract that's being used as the exclusive
    /// controller of this one. Operator is the only party authorized to perform
    /// swaps on users' behalves.
    pub operator_addr: Option<Addr>,

    pub base_token: BaseTokenInitArgs,
    pub base_reserve: Uint128,

    pub quote_token: QuoteTokenInitArgs,
    pub quote_reserve: Uint128,

    pub taker_fee_pct: Uint128,
    pub maker_fee_pct: Uint128,
    pub fee_addr: Addr,
}

#[cw_serde]
pub struct BalanceChangeMsg {
    pub event: BalanceChangeEvent,
}

#[cw_serde]
#[derive(cw_orch::ExecuteFns)]
pub enum ExecuteMsg {
    SetConfig(Config),
    Receive(Cw20ReceiveMsg),
    OnBalanceChange(BalanceChangeMsg),
    Buy(BuyMsg),
}

#[cw_serde]
#[derive(cw_orch::QueryFns, QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},

    #[returns(OverviewResponse)]
    Overview {},

    #[returns(AccountResponse)]
    Account { address: Addr },
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct ConfigResponse(pub Config);

#[cw_serde]
pub struct CurveFeeOverview {
    pub recipient: Addr,
    pub taker_pct: Uint128,
    pub maker_pct: Uint128,
}

#[cw_serde]
pub struct CurveStatsOverview {
    pub bids: SwapStats,
    pub asks: SwapStats,
    pub net_maker_fee: Uint128,
    pub net_taker_fee: Uint128,
}

#[cw_serde]
pub struct CurveAmmOverview {
    pub base_token: Token,
    pub base_reserve: Uint128,
    pub base_decimals: u8,
    pub quote_token: Token,
    pub quote_reserve_real: Uint128,
    pub quote_reserve_virtual: Uint128,
    pub quote_decimals: u8,
    pub constant_product: Uint256,
}

#[cw_serde]
pub struct OverviewResponse {
    pub fees: CurveFeeOverview,
    pub stats: CurveStatsOverview,
    pub amm: CurveAmmOverview,
}

#[cw_serde]
pub struct AccountResponse {
    pub stats: AccountStats,
}

#[cw_serde]
pub struct BuyMsg {
    pub initiator: Option<Addr>,
    pub min_out_amount: Option<Uint128>,
}

#[cw_serde]
pub struct SellMsg {
    pub initiator: Option<Addr>,
    pub min_out_amount: Option<Uint128>,
}

#[cw_serde]
pub enum Cw20ReceiveInnerMsg {
    Buy(BuyMsg),
    Sell(SellMsg),
}

#[cw_serde]
pub enum BalanceChangeEvent {
    Transfer {
        initiator: Addr,
        recipient: Addr,
        initiator_balance: Uint128,
        recipient_balance: Uint128,
        amount: Uint128,
    },
    Burn {
        initiator: Addr,
        initiator_balance: Uint128,
        amount: Uint128,
    },
    Mint {
        initiator: Addr,
        recipient: Addr,
        recipient_balance: Uint128,
        amount: Uint128,
    },
}
