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
    validate_claim(
        &deps,
        info.clone(),
        eth_address.clone(),
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
