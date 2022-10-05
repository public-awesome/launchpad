use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Base(#[from] sg721_base::ContractError),

    #[error("TokenIdNotFound")]
    TokenIdNotFound {},
}
