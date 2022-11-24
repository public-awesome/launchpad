use super::{compute_valid_eth_sig, INSTANTIATION_FEE, MAX_AIRDROP, MIN_AIRDROP, NATIVE_DENOM};
use crate::{
    error::ContractError,
    query::{query_airdrop_is_eligible, query_per_address_limit},
    state::{Config, ADDRS_TO_MINT_COUNT},
};
use cosmwasm_std::{DepsMut, MessageInfo, Uint128};
use cw_utils::must_pay;
use sg_std::Response;

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

pub fn run_validations_for_claim(
    deps: &DepsMut,
    info: MessageInfo,
    eth_address: String,
    eth_sig: String,
    config: Config,
) -> Result<Response, ContractError> {
    validate_is_eligible(deps, eth_address.clone())?;
    validate_eth_sig(deps, info, eth_address.clone(), eth_sig, config)?;
    validate_mints_remaining(deps, &eth_address)?;
    Ok(Response::new())
}

pub fn validate_is_eligible(
    deps: &DepsMut,
    eth_address: String,
) -> Result<Response, ContractError> {
    let eligible = query_airdrop_is_eligible(deps.as_ref(), eth_address.clone())?;
    match eligible {
        true => Ok(Response::new()),
        false => Err(ContractError::AddressNotEligible {
            address: eth_address,
        }),
    }
}

pub fn validate_eth_sig(
    deps: &DepsMut,
    info: MessageInfo,
    eth_address: String,
    eth_sig: String,
    config: Config,
) -> Result<Response, ContractError> {
    let valid_eth_sig = compute_valid_eth_sig(deps, info, &config, eth_sig, eth_address.clone())?;
    match valid_eth_sig {
        true => Ok(Response::new()),
        false => Err(ContractError::AddressNotEligible {
            address: eth_address,
        }),
    }
}

pub fn validate_mints_remaining(
    deps: &DepsMut,
    eth_address: &str,
) -> Result<Response, ContractError> {
    let mint_count = ADDRS_TO_MINT_COUNT.load(deps.storage, eth_address);
    let mint_count = mint_count.unwrap_or(0);
    let per_address_limit = query_per_address_limit(&deps.as_ref())?;
    if mint_count < per_address_limit {
        Ok(Response::new())
    } else {
        Err(ContractError::MintCountReached {
            address: eth_address.to_string(),
        })
    }
}
