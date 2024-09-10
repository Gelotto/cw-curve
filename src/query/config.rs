use crate::{error::ContractError, models::config::Config, msg::ConfigResponse};

use super::ReadonlyContext;

pub fn query_config(ctx: ReadonlyContext) -> Result<ConfigResponse, ContractError> {
    let ReadonlyContext { .. } = ctx;
    Ok(ConfigResponse(Config {}))
}
