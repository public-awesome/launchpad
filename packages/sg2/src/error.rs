use thiserror::Error;

// [FIMXE]: remove as this is not a contract and shouldn't have contract errors
// `StdError` is used for library errors

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("Expected fungible token, received nonfungible")]
    IncorrectFungibility {},
}
