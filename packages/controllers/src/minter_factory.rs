use cosmwasm_std::{ensure_eq, StdError};
use sg2::{msg::UpdateMinterParamsMsg, MinterParams};
use sg_std::NATIVE_DENOM;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum MinterFactoryError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("InstantiateMinterError")]
    InstantiateMinterError {},

    #[error("InvalidDenom")]
    InvalidDenom {},
}

pub fn update_params<T, C>(
    params: &mut MinterParams<C>,
    param_msg: UpdateMinterParamsMsg<T>,
) -> Result<(), MinterFactoryError> {
    params.code_id = param_msg.code_id.unwrap_or(params.code_id);

    if let Some(creation_fee) = param_msg.creation_fee {
        ensure_eq!(
            &creation_fee.denom,
            &NATIVE_DENOM,
            MinterFactoryError::InvalidDenom {}
        );
        params.creation_fee = creation_fee;
    }

    if let Some(min_mint_price) = param_msg.min_mint_price {
        ensure_eq!(
            &min_mint_price.denom,
            &NATIVE_DENOM,
            MinterFactoryError::InvalidDenom {}
        );
        params.min_mint_price = min_mint_price;
    }

    params.mint_fee_bps = param_msg.mint_fee_bps.unwrap_or(params.mint_fee_bps);

    Ok(())
}
