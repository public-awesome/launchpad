use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use sg1::FeeError;
use sg_controllers::MinterFactoryError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Fee(#[from] FeeError),

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("{0}")]
    MinterFactory(#[from] MinterFactoryError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("MinterFactoryError")]
    MinterFactoryError {},
}
