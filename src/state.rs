use cw_storage_plus::{Item, Map};

use cosmwasm_std::{Addr, Response, Uint128};

use crate::{
    curve::Curve,
    error::ContractError,
    execute::Context,
    models::{account::AccountStats, ohlc::OhlcBar},
    msg::InstantiateMsg,
    token::Token,
};

pub const CURVE: Item<Curve> = Item::new("curve");
pub const BUY_FEE_PCT: Item<Uint128> = Item::new("buy_fee_pct");
pub const SELL_FEE_PCT: Item<Uint128> = Item::new("sell_fee_pct");
pub const FEE_RECIPIENT_ADDR: Item<Addr> = Item::new("fee_recipient_addr");
pub const OPERATOR_ADDR: Item<Addr> = Item::new("operator_addr");
pub const QUOTE_TOKEN: Item<Token> = Item::new("quote_token");
pub const QUOTE_RESERVE: Item<Uint128> = Item::new("quote_reserve");
pub const QUOTE_SYMBOL: Item<u8> = Item::new("quote_symbol");
pub const BASE_TOKEN: Item<Token> = Item::new("base_token");
pub const BASE_RESERVE: Item<Uint128> = Item::new("base_reserve");
pub const BASE_SYMBOL: Item<u8> = Item::new("base_symbol");
pub const OHLC_BARS: Map<u64, OhlcBar> = Map::new("ohlc_bars");
pub const ACCOUNT_STATS: Map<&Addr, AccountStats> = Map::new("account_stats");

/// Top-level initialization of contract state
pub fn init(
    _ctx: Context,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("action", "instantiate"))
}

// TODO: Add total cost factor for PnL calculation
// TODO: impl state::init
