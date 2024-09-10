use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("InsufficientFunds: Expected {exp_amount} {denom} but got {amount}")]
    InsufficientFunds {
        denom: String,
        amount: u128,
        exp_amount: u128,
    },

    #[error("MissingFunds: Expected {denom} in funds")]
    MissingFunds { denom: String },

    #[error("NotAuthorized: {reason:?}")]
    NotAuthorized { reason: String },

    #[error("ValidationError: {reason:?}")]
    ValidationError { reason: String },

    #[error("TooMuchSlippage: Exceeded slippage tolerance")]
    TooMuchSlippage {},
}

impl From<ContractError> for StdError {
    fn from(err: ContractError) -> Self {
        StdError::generic_err(err.to_string())
    }
}
