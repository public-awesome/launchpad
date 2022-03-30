use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use sg_std::FeeError;
use thiserror::Error;
use url::ParseError;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

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

    #[error("Invalid Royalities")]
    InvalidRoyalities {},

    #[error("Description too long")]
    DescriptionTooLong {},

    #[error("Cannot update token uri after contract freeze")]
    Frozen {},

    #[error("{0}")]
    BaseError(cw721_base::ContractError),

    #[error["Token id not found {got}"]]
    TokenNotFound { got: String },

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("{0}")]
    Fee(#[from] FeeError),

    #[error("{0}")]
    Parse(#[from] ParseError),
}

impl From<cw721_base::ContractError> for ContractError {
    fn from(err: cw721_base::ContractError) -> Self {
        match err {
            cw721_base::ContractError::Unauthorized {} => Self::Unauthorized {},
            cw721_base::ContractError::Claimed {} => Self::Claimed {},
            cw721_base::ContractError::Expired {} => Self::Expired {},
            err => Self::BaseError(err),
        }
    }
}
