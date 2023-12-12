use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use sg_std::fees::FeeError;
use thiserror::Error;
use url::ParseError;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("TokenMetadataFrozen")]
    TokenMetadataFrozen {},

    #[error("TokenIdNotFound")]
    TokenIdNotFound {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("InvalidCreationFee")]
    InvalidCreationFee {},

    #[error("token_id already claimed")]
    Claimed {},

    #[error("Cannot set approval that is already expired")]
    Expired {},

    #[error("Approval not found for: {spender}")]
    ApprovalNotFound { spender: String },

    #[error("Description too long")]
    DescriptionTooLong {},

    #[error("InvalidRoyalties: {0}")]
    InvalidRoyalties(String),

    #[error("Royalty share percentage can only be altered a day after the last update")]
    RoyaltyUpdateTooSoon {},

    #[error("Royalty Info invalid")]
    RoyaltyInfoInvalid {},

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("{0}")]
    Fee(#[from] FeeError),

    #[error("{0}")]
    Parse(#[from] ParseError),

    #[error("{0}")]
    Base(#[from] cw721_base::ContractError),
}
