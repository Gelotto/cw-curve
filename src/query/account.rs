use cosmwasm_std::Addr;

use crate::{error::ContractError, msg::AccountResponse, state::ACCOUNT_STATS};

use super::ReadonlyContext;

pub fn query_account(
    ctx: ReadonlyContext,
    address: Addr,
) -> Result<AccountResponse, ContractError> {
    let ReadonlyContext { deps, .. } = ctx;
    Ok(AccountResponse {
        stats: ACCOUNT_STATS.load(deps.storage, &deps.api.addr_validate(address.as_str())?)?,
    })
}
