use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("InvalidCreationFee")]
    InvalidCreationFee {},

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("MembersExceeded: {expected} got {actual}")]
    MembersExceeded { expected: u32, actual: u32 },
}
