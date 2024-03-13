use crate::state::{ADDRS_TO_MINT_COUNT, HAS_CLAIMED};
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
    HAS_CLAIMED.save(deps.storage, &eth_address, &true)?;
    Ok(res.add_attribute("claimed_amount", config.airdrop_amount.to_string()))
}
