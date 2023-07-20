use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use thiserror::Error;
use url::ParseError;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("{0}")]
    Parse(#[from] ParseError),

    #[error("{0}")]
    Base(#[from] cw721_base::ContractError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Unauthorized Owner Does Not Match Sender")]
    UnauthorizedOwner {},

    #[error("InvalidCreationFee")]
    InvalidCreationFee {},

    #[error("token_id already claimed")]
    Claimed {},

    #[error("Cannot set approval that is already expired")]
    Expired {},

    #[error("Approval not found for: {spender}")]
    ApprovalNotFound { spender: String },

    #[error("InvalidRoyalties: {0}")]
    InvalidRoyalties(String),

    #[error("Description too long")]
    DescriptionTooLong {},

    #[error("InvalidStartTradingTime")]
    InvalidStartTradingTime {},

    #[error("CollectionInfoFrozen")]
    CollectionInfoFrozen {},

    #[error("MinterNotFound")]
    MinterNotFound {},

    #[error("Ownership Update Error: {error}")]
    OwnershipUpdateError { error: String },

    #[error("Error while migrating: ({0}) ")]
    MigrationError(String),
}
