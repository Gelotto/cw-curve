pub mod buy_sell;
pub mod cw20_receive;
pub mod on_balance_change;
pub mod set_config;

use cosmwasm_std::{DepsMut, Env, MessageInfo};

pub struct Context<'a> {
    pub deps: DepsMut<'a>,
    pub env: Env,
    pub info: MessageInfo,
}
