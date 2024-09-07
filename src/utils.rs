use cosmwasm_std::{Addr, Api, Storage};

use crate::{error::ContractError, state::OPERATOR_ADDR};

// If an operator address exists, then ensure that the only authorized sender is
// the operator itself. The operator is indended to be the address of some other
// smart contract that acts as the exclusive controller for performing swaps.
// This allows the operator to guard or augment swap executions based on its own
// business logic.
pub fn resolve_swap_initiator(
    store: &dyn Storage,
    api: &dyn Api,
    sender: &Addr,
    maybe_initiator: Option<Addr>,
    action: &str,
) -> Result<Addr, ContractError> {
    if let Some(operator_addr) = OPERATOR_ADDR.may_load(store)? {
        if *sender != operator_addr {
            return Err(ContractError::NotAuthorized {
                reason: format!("only operator {} is authorized to {}", operator_addr, action),
            });
        }
        Ok(if let Some(initiator) = maybe_initiator {
            api.addr_validate(initiator.as_str())?
        } else {
            sender.clone()
        })
    } else {
        if maybe_initiator.is_some() {
            return Err(ContractError::NotAuthorized {
                reason: format!("Only an operator can {} but none is set", action),
            });
        }
        Ok(sender.clone())
    }
}
