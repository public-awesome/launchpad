use cosmwasm_std::{Coin, OverflowError, StdError, Timestamp};
use cw_utils::PaymentError;
use thiserror::Error;
use url::ParseError;

use sg1::FeeError;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("{0}")]
    ParseError(#[from] ParseError),

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("{0}")]
    Fee(#[from] FeeError),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("UpdateStatus")]
    UpdateStatus {},

    #[error("Invalid reply ID")]
    InvalidReplyID {},

    #[error("Not enough funds sent")]
    NotEnoughFunds {},

    #[error("TooManyCoins")]
    TooManyCoins {},

    #[error("IncorrectPaymentAmount {0} != {1}")]
    IncorrectPaymentAmount(Coin, Coin),

    #[error("InvalidNumTokens {max}, min: 1")]
    InvalidNumTokens { max: u32, min: u32 },

    #[error("Sold out")]
    SoldOut {},

    #[error("Not Sold out")]
    NotSoldOut {},

    #[error("MintingHasNotYetEnded")]
    MintingHasNotYetEnded {},

    #[error("InvalidDenom {expected} got {got}")]
    InvalidDenom { expected: String, got: String },

    #[error("Minimum network mint price {expected} got {got}")]
    InsufficientMintPrice { expected: u128, got: u128 },

    #[error("Minimum whitelist mint price {expected} got {got}")]
    InsufficientWhitelistMintPrice { expected: u128, got: u128 },

    #[error("Update price {updated} higher than allowed price {allowed}")]
    UpdatedMintPriceTooHigh { allowed: u128, updated: u128 },

    #[error("Invalid address {addr}")]
    InvalidAddress { addr: String },

    #[error("Invalid token id")]
    InvalidTokenId {},

    #[error("AlreadyStarted")]
    AlreadyStarted {},

    #[error("BeforeGenesisTime")]
    BeforeGenesisTime {},

    #[error("WhitelistAlreadyStarted")]
    WhitelistAlreadyStarted {},

    #[error("InvalidStartTime {0} < {1}")]
    InvalidStartTime(Timestamp, Timestamp),

    #[error("InvalidEndTime {0} < {1}")]
    InvalidEndTime(Timestamp, Timestamp),

    #[error("InvalidStartTradingTime {0} > {1}")]
    InvalidStartTradingTime(Timestamp, Timestamp),

    #[error("Instantiate sg721 error")]
    InstantiateSg721Error {},

    #[error("Invalid base token URI (must be an IPFS URI)")]
    InvalidBaseTokenURI {},

    #[error("address not on whitelist: {addr}")]
    NotWhitelisted { addr: String },

    #[error("Minting has not started yet")]
    BeforeMintStartTime {},

    #[error("No End Time Initially Defined")]
    NoEndTimeInitiallyDefined {},

    #[error("Minting has ended")]
    AfterMintEndTime {},

    #[error("Invalid minting limit per address. max: {max}, min: 1, got: {got}")]
    InvalidPerAddressLimit { max: u32, min: u32, got: u32 },

    #[error("Max minting limit per address exceeded")]
    MaxPerAddressLimitExceeded {},

    #[error("Token id: {token_id} already sold")]
    TokenIdAlreadySold { token_id: u32 },

    #[error("NoEnvTransactionIndex")]
    NoEnvTransactionIndex {},

    #[error("Multiply Fraction Error")]
    CheckedMultiplyFractionError {},
}
