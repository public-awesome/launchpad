use cosmwasm_std::StdError;
use cw721::error::Cw721ContractError;
use cw_utils::PaymentError;
use sg1::FeeError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Cw721(#[from] Cw721ContractError),

    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("{0}")]
    Base(#[from] sg721_base::ContractError),

    #[error("{0}")]
    Fee(#[from] FeeError),

    #[error("TokenMetadataFrozen")]
    TokenMetadataFrozen {},

    #[error("NotEnableUpdatable")]
    NotEnableUpdatable {},

    #[error("AlreadyEnableUpdatable")]
    AlreadyEnableUpdatable {},
}
