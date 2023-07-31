use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use sg1::FeeError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Fee(#[from] FeeError),

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("InvalidDenom")]
    InvalidDenom {},

    #[error("Invalid Creation Fee amount received to create the minter.")]
    InvalidCreationFeeAmount {},

    #[error("Factory frozen. Cannot make new minters.")]
    Frozen {},

    #[error("InvalidCollectionCodeId {code_id}")]
    InvalidCollectionCodeId { code_id: u64 },
}
