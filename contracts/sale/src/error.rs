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
}
