use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp, Uint128, Uint64};

#[cw_serde]
pub struct Account {
    pub stats: AccountStats,
}

#[cw_serde]
#[derive(Default)]
pub struct AccountStats {
    pub n_buys: u32,
    pub n_sells: u32,
    pub total_cost: Uint128,
    pub net_quote_in: Uint128,
    pub net_quote_out: Uint128,
    pub net_base_in: Uint128,
    pub net_base_out: Uint128,
}

#[cw_serde]
pub struct MaxSwapInfo {
    pub initiator: Addr,
    pub amount: Uint128,
    pub time: Timestamp,
}

#[cw_serde]
#[derive(Default)]
pub struct SwapStats {
    pub n: Uint64,
    pub max: Option<MaxSwapInfo>,
}
