use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid reply ID")]
    InvalidReplyID {},

    #[error("Not enough funds sent")]
    NotEnoughFunds {},

    #[error("Sold out")]
    SoldOut {},

    #[error("Instantiate sg721 error")]
    InstantiateSg721Error {},

    #[error("Token URI list exceeds max length")]
    MaxTokenURIsLengthExceeded {},

    #[error("Token URI list length does not match number of tokens")]
    TokenURIsListInvalidNumber {},

    #[error("address not on whitelist")]
    NotWhitelisted {},

    #[error("Max whitelist addresses list exceeds max length")]
    MaxWhitelistAddressLengthExceeded {},

    #[error("Minting has not started yet")]
    BeforeMintStartTime {},

    #[error("Invalid minting limit per address. Max limit is 30.")]
    InvalidPerAddressLimit {},

    #[error("Max minting limit per address exceeded")]
    MaxPerAddressLimitExceeded {},
}
