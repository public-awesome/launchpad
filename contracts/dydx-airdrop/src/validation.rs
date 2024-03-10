use cosmwasm_std::{DepsMut, MessageInfo, StdResult};
use crate::ContractError;
use crate::state::{ADDRS_TO_MINT_COUNT};
use crate::state::IS_ADDRESS_REGISTERED;

use cosmwasm_std::StdError;
use ethereum_verify::verify_ethereum_text;

use crate::{
    query::{query_airdrop_is_eligible, query_per_address_limit},
    state::Config,
};

pub fn compute_plaintext_msg(config: &Config, info: MessageInfo) -> String {
    str::replace(
        &config.claim_msg_plaintext,
        "{wallet}",
        info.sender.as_ref(),
    )
}

pub fn validate_registration(
    deps: &DepsMut,
    info: MessageInfo,
    eth_address: String,
    eth_sig: String,
    config: Config,
) -> Result<(), ContractError> {
    validate_is_eligible(deps, eth_address.clone())?;
    validate_eth_sig(deps, info, eth_address.clone(), eth_sig, config)?;
    check_previous_registration(deps, &eth_address)?;
    Ok(())
}

pub fn validate_claim(
    deps: &DepsMut,
    info: MessageInfo,
    eth_address: String,
    eth_sig: String,
    config: Config,
) -> Result<(), ContractError> {
    validate_is_eligible(deps, eth_address.clone())?;
    validate_eth_sig(deps, info, eth_address.clone(), eth_sig, config)?;
    validate_mints_remaining(deps, &eth_address)?;
    Ok(())
}

fn validate_is_eligible(deps: &DepsMut, eth_address: String) -> Result<(), ContractError> {
    let eligible = query_airdrop_is_eligible(deps.as_ref(), eth_address.clone())?;
    match eligible {
        true => Ok(()),
        false => Err(ContractError::AddressNotEligible {
            address: eth_address,
        }),
    }
}

fn validate_eth_sig(
    deps: &DepsMut,
    info: MessageInfo,
    eth_address: String,
    eth_sig: String,
    config: Config,
) -> Result<(), ContractError> {
    let valid_eth_sig =
        validate_ethereum_text(deps, info, &config, eth_sig, eth_address.clone())?;
    match valid_eth_sig {
        true => Ok(()),
        false => Err(ContractError::AddressNotEligible {
            address: eth_address,
        }),
    }
}

pub fn validate_mints_remaining(
    deps: &DepsMut,
    eth_address: &str,
) -> Result<(), ContractError> {
    let mint_count = ADDRS_TO_MINT_COUNT.load(deps.storage, eth_address);
    let mint_count = mint_count.unwrap_or(0);
    let per_address_limit = query_per_address_limit(&deps.as_ref())?;
    if mint_count < per_address_limit {
        Ok(())
    } else {
        Err(ContractError::MintCountReached {
            address: eth_address.to_string(),
        })
    }
}

pub fn validate_ethereum_text(
    deps: &DepsMut,
    info: MessageInfo,
    config: &Config,
    eth_sig: String,
    eth_address: String,
) -> StdResult<bool> {
    let plaintext_msg = compute_plaintext_msg(config, info);
    match hex::decode(eth_sig.clone()) {
        Ok(eth_sig_hex) => {
            verify_ethereum_text(deps.as_ref(), &plaintext_msg, &eth_sig_hex, &eth_address)
        }
        Err(_) => Err(StdError::InvalidHex {
            msg: format!("Could not decode {eth_sig}"),
        }),
    }
}

pub fn check_previous_registration(
    deps: &DepsMut,
    eth_address: &str,
) -> Result<(), ContractError> {
    let registered = IS_ADDRESS_REGISTERED.load(deps.storage, eth_address).unwrap_or(false);
    if registered {
        Err(ContractError::AlreadyRegistered {
            address: eth_address.to_string(),
        })
    } else {
        Ok(())
    }
}