use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("Expected fungible token, received nonfungible")]
    IncorrectFungibility {},
}
