use cosmwasm_std::StdError;
use cw_controllers::AdminError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("NoMinting")]
    NoMinting {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Marketplace contract invalid address '{addr}'")]
    InvalidMarketplace { addr: String },

    #[error("{0}")]
    Admin(#[from] AdminError),
}
