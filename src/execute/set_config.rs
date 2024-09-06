use crate::{error::ContractError, models::config::Config};
use cosmwasm_std::{attr, Response};

use super::Context;

pub fn exec_set_config(
    ctx: Context,
    config: Config,
) -> Result<Response, ContractError> {
    let Context { deps, .. } = ctx;
    Ok(Response::new().add_attributes(vec![attr("action", "set_config")]))
}
