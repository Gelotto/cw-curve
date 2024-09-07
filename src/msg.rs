use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

use crate::{models::config::Config, token::Token};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
#[derive(cw_orch::ExecuteFns)]
pub enum ExecuteMsg {
    SetConfig(Config),
    Buy(SwapMsg),
    Sell(SwapMsg),
}

#[cw_serde]
#[derive(cw_orch::QueryFns, QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct ConfigResponse(pub Config);

#[cw_serde]
pub struct SwapMsg {
    pub initiator: Option<Addr>,
    pub token: Token,
    pub amount: Uint128,
    pub min_out_amount: Option<Uint128>,
}
