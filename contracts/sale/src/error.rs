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
}

impl From<ParseError> for ContractError {
    fn from(_err: ParseError) -> ContractError {
        ContractError::InvalidBaseTokenURI {}
    }
}
