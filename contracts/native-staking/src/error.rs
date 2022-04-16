use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("DelegationNotFound")]
    DelegationNotFound {},

    #[error("MinDurationNotPassed")]
    MinDurationNotPassed {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },

    #[error("{0}")]
    Payment(#[from] PaymentError),
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
