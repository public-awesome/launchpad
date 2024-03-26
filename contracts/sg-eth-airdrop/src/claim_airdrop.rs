use crate::query::query_collection_whitelist;
use crate::state::ADDRS_TO_MINT_COUNT;
use crate::{state::CONFIG, ContractError};
use build_messages::claim_and_whitelist_add;
use cosmwasm_std::DepsMut;
use cosmwasm_std::{coins, Addr, BankMsg, CosmosMsg, Response, StdResult, SubMsg};
use cosmwasm_std::{Env, MessageInfo};

use sg_whitelist::msg::ExecuteMsg as CollectionWhitelistExecuteMsg;
use sg_whitelist::{helpers::interface::CollectionWhitelistContract, msg::AddMembersMsg};
use validation::validate_claim;

pub fn claim_airdrop(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    eth_address: String,
    eth_sig: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    validate_claim(
        &deps,
        info.clone(),
        eth_address.clone(),
        eth_sig,
        config.clone(),
    )?;
    let res = claim_and_whitelist_add(&deps, info, config.airdrop_amount)?;
    increment_local_mint_count_for_address(deps, eth_address)?;

    Ok(res.add_attribute("claimed_amount", config.airdrop_amount.to_string()))
}

pub fn increment_local_mint_count_for_address(
    deps: DepsMut,
    eth_address: String,
) -> Result<Response, ContractError> {
    let mint_count_for_address = ADDRS_TO_MINT_COUNT
        .load(deps.storage, &eth_address)
        .unwrap_or(0);
    ADDRS_TO_MINT_COUNT.save(deps.storage, &eth_address, &(mint_count_for_address + 1))?;

    Ok(Response::new())
}

mod build_messages {
    use super::*;
    use sg_std::NATIVE_DENOM;

    pub fn claim_and_whitelist_add(
        deps: &DepsMut,
        info: MessageInfo,
        airdrop_amount: u128,
    ) -> Result<Response, ContractError> {
        let mut res = Response::new();
        let bank_msg = SubMsg::new(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: coins(airdrop_amount, NATIVE_DENOM),
        });
        res = res.add_submessage(bank_msg);
        let collection_whitelist = query_collection_whitelist(deps)?;
        let res = res.add_message(add_member_to_collection_whitelist(
            deps,
            info.sender,
            collection_whitelist,
        )?);
        Ok(res)
    }

    fn add_member_to_collection_whitelist(
        deps: &DepsMut,
        wallet_address: Addr,
        collection_whitelist: String,
    ) -> StdResult<CosmosMsg> {
        let inner_msg = AddMembersMsg {
            to_add: vec![wallet_address.to_string()],
        };
        let execute_msg = CollectionWhitelistExecuteMsg::AddMembers(inner_msg);
        CollectionWhitelistContract(deps.api.addr_validate(&collection_whitelist)?)
            .call(execute_msg)
    }
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
}
