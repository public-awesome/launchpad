use cosmwasm_std::{DepsMut, MessageInfo, StdResult};
use crate::ContractError;
use crate::state::{ADDRS_TO_MINT_COUNT};
use crate::state::IS_ADDRESS_REGISTERED;

use cosmwasm_std::StdError;
use ethereum_verify::verify_ethereum_text;

use crate::{
    query::{query_airdrop_is_eligible},
    state::Config,
};

use cosmwasm_std::Uint128;
use cw_utils::must_pay;
use sg_std::NATIVE_DENOM;
use crate::contract::INSTANTIATION_FEE;
use crate::msg::InstantiateMsg;

const MIN_AIRDROP: u128 = 10_000_000; // 10 STARS
const MAX_AIRDROP: u128 = 100_000_000_000_000; // 100 million STARS

pub fn validate_instantiation_params(
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<(), ContractError> {
    validate_airdrop_amount(msg.airdrop_amount)?;
    validate_plaintext_msg(msg.claim_msg_plaintext)?;
    validate_instantiate_funds(info)?;
    Ok(())
}

pub fn validate_instantiate_funds(info: MessageInfo) -> Result<(), ContractError> {
    let amount = must_pay(&info, NATIVE_DENOM)?;
    if amount < Uint128::from(INSTANTIATION_FEE) {
        return Err(ContractError::InsufficientFundsInstantiate {});
    };
    Ok(())
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

pub fn validate_plaintext_msg(plaintext_msg: String) -> Result<(), ContractError> {
    if !plaintext_msg.contains("{wallet}") {
        return Err(ContractError::PlaintextMsgNoWallet {});
    }
    if plaintext_msg.len() > 1000 {
        return Err(ContractError::PlaintextTooLong {});
    }
    Ok(())
}

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
    // TODO: Replace with collection and names mint validation
    // validate_mints_remaining(deps, &eth_address)?;
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

// TODO: Implement validate_collection_mint for reward claim
pub fn validate_collection_mint(
    deps: &DepsMut,
    eth_address: &str,
) -> Result<(), ContractError> {
    unimplemented!();
    // let mint_count = ADDRS_TO_MINT_COUNT.load(deps.storage, eth_address);
    // let mint_count = mint_count.unwrap_or(0);
    //let per_address_limit = query_per_address_limit(&deps.as_ref())?;
    // if mint_count < per_address_limit {
    //     Ok(())
    // } else {
    //     Err(ContractError::MintCountReached {
    //         address: eth_address.to_string(),
    //     })
    // }
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

pub fn validate_required_action_completion(
    deps: &DepsMut,
    eth_address: &str,
) -> Result<(), ContractError> {
   unimplemented!()
    // validate_collection_mint()
    // validate_names_mint()
}