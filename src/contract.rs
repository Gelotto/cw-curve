use crate::error::ContractError;
use crate::execute::buy_sell::{exec_buy, exec_sell};
use crate::execute::cw20_receive::exec_cw20_receive;
use crate::execute::on_balance_change::exec_on_balance_change;
use crate::execute::{set_config::exec_set_config, Context};
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::query::account::query_account;
use crate::query::overview::query_overview;
use crate::query::{config::query_config, ReadonlyContext};
use crate::state::{self, handle_cw20_instantiate_reply, CW20_INSTANTIATE_REPLY_ID};
use cosmwasm_std::{entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = "crates.io:cw-curve";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(state::init(Context { deps, env, info }, msg)?)
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let ctx = Context { deps, env, info };
    match msg {
        ExecuteMsg::SetConfig(config) => exec_set_config(ctx, config),
        ExecuteMsg::OnBalanceChange(msg) => exec_on_balance_change(ctx, msg),
        ExecuteMsg::Receive(msg) => exec_cw20_receive(ctx, msg),
        ExecuteMsg::Buy(msg) => exec_buy(ctx, msg, None),
    }
}

#[entry_point]
pub fn reply(
    deps: DepsMut,
    _env: Env,
    reply: Reply,
) -> Result<Response, ContractError> {
    if reply.id == CW20_INSTANTIATE_REPLY_ID {
        handle_cw20_instantiate_reply(deps, reply)?;
    }
    Ok(Response::new())
}

#[entry_point]
pub fn query(
    deps: Deps,
    env: Env,
    msg: QueryMsg,
) -> Result<Binary, ContractError> {
    let ctx = ReadonlyContext { deps, env };
    let result = match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(ctx)?),
        QueryMsg::Overview {} => to_json_binary(&query_overview(ctx)?),
        QueryMsg::Account { address } => to_json_binary(&query_account(ctx, address)?),
    }?;
    Ok(result)
}

#[entry_point]
pub fn migrate(
    deps: DepsMut,
    _env: Env,
    _msg: MigrateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
