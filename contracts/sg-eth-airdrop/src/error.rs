use cosmwasm_std::{Addr, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    // #[error("Unauthorized")]
    // Unauthorized {},
    #[error("Contract has no funds")]
    NoFunds {},

    #[error("Invalid reply ID")]
    InvalidReplyID {},

    // #[error("Sender {sender} is not an admin")]
    // Unauthorized { sender: Addr},
    #[error("Unauthorized admin, sender is {sender}")]
    Unauthorized { sender: Addr },

    #[error("Reply error")]
    ReplyOnSuccess {},
}
