use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use sg1::FeeError;
use sg_controllers::MinterFactoryError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Fee(#[from] FeeError),

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("{0}")]
    MinterFactory(#[from] MinterFactoryError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("MinterFactoryError")]
    MinterFactoryError {},

    #[error("InvalidNumTokens {max}, min: 1")]
    InvalidNumTokens { max: u32, min: u32 },

    #[error("Invalid minting limit per address. max: {max}, min: 1, got: {got}")]
    InvalidPerAddressLimit { max: u32, min: u32, got: u32 },

    #[error("InvalidDenom")]
    InvalidDenom {},

    #[error("Minimum network mint price {expected} got {got}")]
    InsufficientMintPrice { expected: u128, got: u128 },

    #[error("Invalid reply ID")]
    InvalidReplyID {},
    // #[error("Custom Error val: {val:?}")]
    // CustomError { val: String },
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
