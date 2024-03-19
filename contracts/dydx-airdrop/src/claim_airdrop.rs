use crate::build_messages::claim_reward;
use crate::state::{AIRDROP_COUNT, HAS_CLAIMED};
use crate::{state::CONFIG, ContractError};
use cosmwasm_std::DepsMut;
use cosmwasm_std::{Env, MessageInfo};
use sg_std::Response;

use crate::validation::validate_claim;

pub fn claim_airdrop(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    eth_address: String,
    eth_sig: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let airdrop_count_limit = config.airdrop_count_limit;
    if AIRDROP_COUNT.load(deps.storage)? >= airdrop_count_limit {
        return Err(ContractError::AirdropCountLimitExceeded {});
    }

    let eth_address_lower = eth_address.to_ascii_lowercase();
    validation::assert_lower_case(eth_address_lower.clone())?;

    validate_claim(
        &deps,
        info.clone(),
        eth_address_lower.clone(),
        eth_sig,
        config.clone(),
    )?;

    let res = claim_reward(info, config.airdrop_amount)?;
    AIRDROP_COUNT.update(deps.storage, |count: u32| -> Result<u32, ContractError> {
        Ok(count + 1)
    })?;
    HAS_CLAIMED.save(deps.storage, &eth_address, &true)?;
    Ok(res.add_attribute("claimed_amount", config.airdrop_amount.to_string()))
}

mod validation {
    use super::*;
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

    pub fn assert_lower_case(eth_address: String) -> Result<(), ContractError> {
        match eth_address.to_lowercase() == eth_address {
            true => Ok(()),
            false => Err(ContractError::EthAddressShouldBeLower {
                address: eth_address,
            }),
        }
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
}
