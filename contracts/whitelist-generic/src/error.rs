use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    PaymentError(#[from] PaymentError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("AddressNotFound {addr}")]
    AddressNotFound { addr: String },

    #[error("OverPerAddressLimit")]
    OverPerAddressLimit {},

    #[error("AddressAlreadyExists {addr}")]
    AddressAlreadyExists { addr: String },
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
