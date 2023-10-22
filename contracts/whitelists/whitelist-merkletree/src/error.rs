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

    #[error("AlreadyEnded")]
    AlreadyEnded {},

    #[error("InvalidDenom: {0}")]
    InvalidDenom(String),

    #[error("NoMemberFound: {0}")]
    NoMemberFound(String),

    #[error("InvalidStartTime {0} > {1}")]
    InvalidStartTime(Timestamp, Timestamp),

    #[error("InvalidEndTime {0} > {1}")]
    InvalidEndTime(Timestamp, Timestamp),

    #[error("Invalid merkle tree URI (must be an IPFS URI)")]
    InvalidMerkleTreeURI {},

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

    #[error("InvalidHashString: {0}")]
    InvalidHashString(String),
}
