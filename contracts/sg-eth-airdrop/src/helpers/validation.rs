use super::{INSTANTIATION_FEE, MAX_AIRDROP, MIN_AIRDROP, NATIVE_DENOM};
use crate::error::ContractError;
use cosmwasm_std::{MessageInfo, Uint128};
use cw_utils::must_pay;

pub fn check_instantiate_funds(info: MessageInfo) -> Result<u128, ContractError> {
    let amount = must_pay(&info, NATIVE_DENOM)?;
    if amount < Uint128::from(INSTANTIATION_FEE) {
        return Err(ContractError::InsufficientFundsInstantiate {});
    };
    Ok(amount.u128())
}

pub fn validate_airdrop_amount(airdrop_amount: u128) -> Result<u128, ContractError> {
    if airdrop_amount < MIN_AIRDROP {
        return Err(ContractError::AirdropTooSmall {});
    };
    if airdrop_amount > MAX_AIRDROP {
        return Err(ContractError::AirdropTooBig {});
    };
    Ok(airdrop_amount)
}

pub fn validate_plaintext_msg(airdrop_amount: u128) -> Result<u128, ContractError> {
    if airdrop_amount < MIN_AIRDROP {
        return Err(ContractError::AirdropTooSmall {});
    };
    if airdrop_amount > MAX_AIRDROP {
        return Err(ContractError::AirdropTooBig {});
    };
    Ok(airdrop_amount)
}
