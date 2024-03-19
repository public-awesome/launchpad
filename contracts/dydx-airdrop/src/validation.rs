use crate::state::CONFIG;
use crate::state::{HAS_CLAIMED, IS_ADDRESS_REGISTERED};
use crate::ContractError;
use cosmwasm_std::{DepsMut, MessageInfo, StdResult};

use cosmwasm_std::StdError;
use ethereum_verify::verify_ethereum_text;

use crate::{query::query_airdrop_is_eligible, state::Config};

use crate::contract::INSTANTIATION_FEE;
use crate::msg::InstantiateMsg;
use crate::query::query_collection_address;
use cosmwasm_std::Uint128;
use cw721::TokensResponse;
use cw_utils::must_pay;
use sg721_base::msg::QueryMsg;
use sg721_name::msg::QueryMsg as NameQueryMsg;
use sg_std::NATIVE_DENOM;

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

pub fn assert_lower_case(eth_address: String) -> Result<(), ContractError> {
    match eth_address.to_lowercase() == eth_address {
        true => Ok(()),
        false => Err(ContractError::EthAddressShouldBeLower {
            address: eth_address,
        }),
    }
}

pub fn validate_registration(
    deps: &DepsMut,
    info: MessageInfo,
    eth_address: String,
    eth_sig: String,
    config: Config,
) -> Result<(), ContractError> {
    assert_lower_case(eth_address.clone())?;
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
    assert_lower_case(eth_address.clone())?;
    validate_is_eligible(deps, eth_address.clone())?;
    validate_eth_sig(deps, info.clone(), eth_address.clone(), eth_sig, config)?;
    check_previous_claim(deps, &eth_address.clone())?;
    validate_collection_mint(deps, info.clone())?;
    validate_name_mint_and_association(deps, info.clone())?;
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
    let valid_eth_sig = validate_ethereum_text(deps, info, &config, eth_sig, eth_address.clone())?;
    match valid_eth_sig {
        true => Ok(()),
        false => Err(ContractError::AddressNotEligible {
            address: eth_address,
        }),
    }
}

pub fn validate_collection_mint(deps: &DepsMut, info: MessageInfo) -> Result<(), ContractError> {
    let collection_address = query_collection_address(deps)?;
    let tokens_response: TokensResponse = deps.querier.query_wasm_smart(
        collection_address,
        &QueryMsg::Tokens {
            owner: String::from(info.sender.clone()),
            start_after: None,
            limit: None,
        },
    )?;
    if tokens_response.tokens.is_empty() {
        return Err(ContractError::CollectionNotMinted {});
    }
    Ok(())
}

pub fn validate_name_mint_and_association(
    deps: &DepsMut,
    info: MessageInfo,
) -> Result<(), ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let name_collection_address = config.name_collection_address;
    let tokens_response: TokensResponse = deps.querier.query_wasm_smart(
        name_collection_address.clone(),
        &QueryMsg::Tokens {
            owner: String::from(info.sender.clone()),
            start_after: None,
            limit: None,
        },
    )?;
    if tokens_response.tokens.is_empty() {
        return Err(ContractError::NameNotMinted {});
    };
    let _associated_name: String = deps.querier.query_wasm_smart(
        name_collection_address.clone(),
        &NameQueryMsg::Name {
            address: String::from(info.sender.clone()),
        },
    )?;
    Ok(())
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

pub fn check_previous_registration(deps: &DepsMut, eth_address: &str) -> Result<(), ContractError> {
    let registered = IS_ADDRESS_REGISTERED
        .load(deps.storage, eth_address)
        .unwrap_or(false);
    if registered {
        return Err(ContractError::AlreadyRegistered {
            address: eth_address.to_string(),
        });
    }
    Ok(())
}

pub fn check_previous_claim(deps: &DepsMut, eth_address: &str) -> Result<(), ContractError> {
    let already_claimed = HAS_CLAIMED.load(deps.storage, eth_address).unwrap_or(false);
    if already_claimed {
        return Err(ContractError::AlreadyClaimed {
            address: eth_address.to_string(),
        });
    }
    Ok(())
}
