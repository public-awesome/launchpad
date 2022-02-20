use cosmwasm_std::StdError;
use cw4_group::ContractError as Cw4GroupContractError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("{0}")]
    C4ContractError(#[from] Cw4GroupContractError),
}
