use crate::state::ADDRS_TO_MINT_COUNT;
use crate::{state::CONFIG, ContractError};
use cosmwasm_std::DepsMut;
use cosmwasm_std::{Env, MessageInfo};
use sg_std::Response;
use crate::build_messages::{claim_reward};

use crate::validation::validate_claim;

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
    let res = claim_reward(info, config.airdrop_amount)?;

    // TODO: To be removed
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
