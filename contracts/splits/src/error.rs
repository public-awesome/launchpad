use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Contract has no funds")]
    NoFunds {},

    #[error("Invalid reply ID")]
    InvalidReplyID {},

    #[error("Reply error")]
    ReplyOnSuccess {},

    #[error("Group contract invalid address '{addr}'")]
    InvalidGroup { addr: String },

    #[error("Group contract invalid total weight '{weight}'")]
    InvalidWeight { weight: u64 },
}
