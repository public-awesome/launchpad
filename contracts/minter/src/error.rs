use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},
    #[error("UnknownReplyId")]
    UnknownReplyId { id: u64 },
    #[error("InvalidReplyData")]
    InvalidReplyData {},
}
