use cosmwasm_std::{StdError, Timestamp};
use cw_utils::PaymentError;
use thiserror::Error;

use base_factory::ContractError as BaseContractError;
use sg1::FeeError;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Fee(#[from] FeeError),

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("InvalidStartTime {0} < {1}")]
    InvalidStartTime(Timestamp, Timestamp),

    #[error("InvalidEndTime {0} > {1}")]
    InvalidEndTime(Timestamp, Timestamp),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("LimitOfTimeOrNumTokensRequired")]
    LimitOfTimeOrNumTokensRequired {},

    #[error("InvalidMintPrice")]
    InvalidMintPrice {},

    #[error("InvalidNftDataProvided")]
    InvalidNftDataProvided {},

    #[error("InvalidMintStartTime")]
    InvalidMintStartTime {},

    #[error("InvalidMintEndTime")]
    InvalidMintEndTime {},

    #[error("NftDataAlreadyLoaded")]
    NftDataAlreadyLoaded {},

    #[error("InvalidNumTokens {max}, min: 1")]
    InvalidNumTokens { max: u32, min: u32 },

    #[error("Invalid minting limit per address. max: {max}, min: 1, got: {got}")]
    InvalidPerAddressLimit { max: u32, min: u32, got: u32 },

    #[error("Minimum network mint price {expected} got {got}")]
    InsufficientMintPrice { expected: u128, got: u128 },

    #[error("{0}")]
    BaseError(#[from] BaseContractError),
}
