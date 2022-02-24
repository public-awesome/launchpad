use cosmwasm_std::StdError;
use sg_std::fees::FeeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("MembersExceeded: {expected} got {actual}")]
    MembersExceeded { expected: u32, actual: u32 },

    #[error("{0}")]
    Fee(#[from] FeeError),
}
