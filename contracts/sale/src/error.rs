use cosmwasm_std::StdError;
use thiserror::Error;
use url::ParseError;

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

    #[error("Invalid base token URI (must be an IPFS URI)")]
    InvalidBaseTokenURI {},

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

impl From<ParseError> for ContractError {
    fn from(_err: ParseError) -> ContractError {
        ContractError::InvalidBaseTokenURI {}
    }
}
