use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("NoMinting")]
    NoMinting {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Marketplace not set")]
    MarketplaceNotSet {},

    #[error("Marketplace contract invalid address '{addr}'")]
    InvalidMarketplace { addr: String },
}
