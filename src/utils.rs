use cosmwasm_std::{Addr, Api, Storage, Uint128};

use crate::{error::ContractError, math::mul_ratio_u128, state::OPERATOR_ADDR};

/// Return the tx sender address of the initiator address if exists AND the tx
/// sender is the registered "operator" address.
pub fn resolve_initiator(
    store: &dyn Storage,
    api: &dyn Api,
    sender: &Addr,
    maybe_initiator: Option<Addr>,
) -> Result<Addr, ContractError> {
    if let Some(candidate_initiator) = maybe_initiator {
        if let Some(operator_addr) = OPERATOR_ADDR.may_load(store)? {
            if operator_addr == sender {
                return Ok(api.addr_validate(candidate_initiator.as_str())?);
            }
        }
        Err(ContractError::NotAuthorized {
            reason: "Only contract operator can specify initiator".to_owned(),
        })
    } else {
        Ok(sender.clone())
    }
}

/// Calculates BASE price with respect to QUOTE
pub fn calculate_spot_price(
    base_reserve: Uint128,
    quote_reserve: Uint128,
    quote_decimals: u8,
) -> Result<Uint128, ContractError> {
    mul_ratio_u128(quote_reserve, 10u128.pow(quote_decimals as u32), base_reserve)
}
