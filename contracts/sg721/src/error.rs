use cosmwasm_std::StdError;
use cw721_base::ContractError as Cw721ContractError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("token_id already claimed")]
    Claimed {},

    #[error("Cannot set approval that is already expired")]
    Expired {},

    #[error("Approval not found for: {spender}")]
    ApprovalNotFound { spender: String },

    #[error("Invalid Royalities")]
    InvalidRoyalities {},
}

impl From<ContractError> for Cw721ContractError {
    fn from(err: ContractError) -> Cw721ContractError {
        match err {
            ContractError::Unauthorized {} => Cw721ContractError::Unauthorized {},
            ContractError::Claimed {} => Cw721ContractError::Claimed {},
            ContractError::Expired {} => Cw721ContractError::Expired {},
            _ => unreachable!("cannot convert {:?} to Cw721ContractError", err),
        }
    }
}
