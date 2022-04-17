use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("StakeNotExpired")]
    StakeNotExpired {},

    #[error("BalanceTooSmall")]
    BalanceTooSmall {},

    #[error("{0}")]
    Payment(#[from] PaymentError),
}
