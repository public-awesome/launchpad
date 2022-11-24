#[cfg(not(feature = "library"))]
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{ADDRS_TO_MINT_COUNT, CONFIG};

use crate::helpers::{
    build_config_msg, build_messages_for_claim_and_whitelist_add, build_whitelist_instantiate_msg,
    check_funds_and_fair_burn, run_validations_for_claim, CONTRACT_NAME, CONTRACT_VERSION,
};
use crate::query::query_airdrop_is_eligible;
use cosmwasm_std::{entry_point, Addr};
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, StdResult};
use cw2::set_contract_version;
use sg_std::Response;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let res = check_funds_and_fair_burn(info.clone())?;
    let cfg = build_config_msg(deps.as_ref(), info.clone(), msg.clone())?;
    CONFIG.save(deps.storage, &cfg)?;
    Ok(res
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION)
        .add_attribute("sender", info.sender)
        .add_submessage(build_whitelist_instantiate_msg(env, msg)?))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ClaimAirdrop {
            eth_address,
            eth_sig,
        } => claim_airdrop(deps, info, _env, eth_address, eth_sig),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::AirdropEligible { eth_address } => {
            to_binary(&query_airdrop_is_eligible(deps, eth_address)?)
        }
        QueryMsg::GetMinter {} => to_binary(&get_minter(deps)?),
    }
}

fn claim_airdrop(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    eth_address: String,
    eth_sig: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    run_validations_for_claim(
        &deps,
        info.clone(),
        eth_address.clone(),
        eth_sig,
        config.clone(),
    )?;
    let res = build_messages_for_claim_and_whitelist_add(&deps, info, config.airdrop_amount)?;
    increment_local_mint_count_for_address(deps, eth_address)?;

    Ok(res
        .add_attribute("claimed_amount", config.airdrop_amount.to_string())
        .add_attribute("minter_address", config.minter_address.to_string()))
}

pub fn get_minter(deps: Deps) -> StdResult<Addr> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config.minter_address)
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
