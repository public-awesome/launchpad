use cosmwasm_std::StdError;
use cw_utils::Expiration;
use sg_std::fees::FeeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("InvalidStartTime {0} > {1}")]
    InvalidStartTime(Expiration, Expiration),

    #[error("MembersExceeded: {expected} got {actual}")]
    MembersExceeded { expected: u32, actual: u32 },

    #[error("Invalid minting limit per address. max: {max}, got: {got}")]
    InvalidPerAddressLimit { max: String, got: String },

    #[error("Max minting limit per address exceeded")]
    MaxPerAddressLimitExceeded {},

    #[error("{0}")]
    Fee(#[from] FeeError),

    #[error("InvalidUnitPrice {0} < {1}")]
    InvalidUnitPrice(u128, u128),
}
