use cosmwasm_std::{Addr, StdError};
use cw_utils::{self, PaymentError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    PaymentError(#[from] PaymentError),

    #[error("Contract has no funds")]
    NoFunds {},

    #[error("Insufficient Funds for Instantiate")]
    InsufficientFundsInstantiate {},

    #[error("Airdrop Amount Too Small")]
    AirdropTooSmall {},

    #[error("Airdrop Amount Too Big")]
    AirdropTooBig {},

    #[error("Invalid reply ID")]
    InvalidReplyID {},

    #[error("Unauthorized admin, sender is {sender}")]
    Unauthorized { sender: Addr },

    #[error("Reply error")]
    ReplyOnSuccess {},

    #[error("Whitelist contract has not been set")]
    WhitelistContractNotSet {},

    #[error("Minter already set")]
    MinterAlreadySet {},

    #[error("Address {address} is not eligible")]
    AddressNotEligible { address: String },

    #[error("Address {address} has already claimed all available mints")]
    MintCountReached { address: String },

    #[error("Address {address} has already registered for the airdrop")]
    AlreadyRegistered { address: String },

    #[error("Address {address} has already claimed the airdrop")]
    AlreadyClaimed { address: String },

    #[error("Collection Whitelist on Minter contract has not been set")]
    CollectionWhitelistMinterNotSet {},

    #[error("Minter config is missing the collection contract address")]
    CollectionContractNotSet {},

    #[error("Need to mint a Geckies token first")]
    CollectionNotMinted {},

    #[error("Need to mint a name first")]
    NameNotMinted {},

    #[error("Airdrop count limit exceeded")]
    AirdropCountLimitExceeded {},

    #[error("Plaintext message must contain `{{wallet}}` string")]
    PlaintextMsgNoWallet {},

    #[error("Plaintext message is too long")]
    PlaintextTooLong {},
}
