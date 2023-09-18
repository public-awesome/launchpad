use cosmwasm_std::{StdError, Timestamp};
use cw_utils::PaymentError;
use sg1::FeeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("AlreadyStarted")]
    AlreadyStarted {},

    #[error("DuplicateMember: {0}")]
    DuplicateMember(String),

    #[error("NoMemberFound: {0}")]
    NoMemberFound(String),

    #[error("InvalidStartTime {0} > {1}")]
    InvalidStartTime(Timestamp, Timestamp),

    #[error("InvalidEndTime {0} > {1}")]
    InvalidEndTime(Timestamp, Timestamp),

    #[error("InvalidWhaleCap {0} > {1}")]
    InvalidWhaleCap(u32, u32),

    #[error("MembersExceeded: {expected} got {actual}")]
    MembersExceeded { expected: u32, actual: u32 },

    #[error("Exceeded whale cap")]
    ExceededWhaleCap {},

    #[error("Invalid member limit. min: {min}, max: {max}, got: {got}")]
    InvalidMemberLimit { min: u32, max: u32, got: u32 },

    #[error("Max minting limit per address exceeded")]
    MaxPerAddressLimitExceeded {},

    #[error("{0}")]
    Fee(#[from] FeeError),

    #[error("InvalidUnitPrice {0} < {1}")]
    InvalidUnitPrice(u128, u128),

    #[error("IncorrectCreationFee {0} < {1}")]
    IncorrectCreationFee(u128, u128),

    #[error("{0}")]
    PaymentError(#[from] PaymentError),

    #[error("UnauthorizedAdmin")]
    UnauthorizedAdmin {},
}
