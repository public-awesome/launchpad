#[cfg(not(feature = "library"))]
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::CONFIG;

use crate::helpers::{
    build_config_msg, build_messages_for_claim_and_whitelist_add, build_whitelist_instantiate_msg,
    check_funds_and_fair_burn, compute_valid_eth_sig, get_add_eligible_eth_response, CONTRACT_NAME,
    CONTRACT_VERSION, NATIVE_DENOM,
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
    let whitelist_instantiate_msg = build_whitelist_instantiate_msg(env, msg)?;

    Ok(res
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION)
        .add_attribute("sender", info.sender)
        .add_submessage(whitelist_instantiate_msg))
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
        ExecuteMsg::AddEligibleEth { eth_addresses } => {
            get_add_eligible_eth_response(deps, info, eth_addresses)
        }
        ExecuteMsg::UpdateMinterAddress { minter_address } => {
            update_minter(deps, info, minter_address)
        }
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
    env: Env,
    eth_address: String,
    eth_sig: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let is_eligible = query_airdrop_is_eligible(deps.as_ref(), eth_address.clone())?;
    let valid_eth_signature =
        compute_valid_eth_sig(&deps, info.clone(), &config, eth_sig, eth_address.clone())?;

    println!(
        "OWN BALANCE {:?}",
        deps.querier
            .query_balance(env.contract.address, NATIVE_DENOM)
    );
    let (mut res, mut claimed_amount) = (Response::new(), 0);
    if is_eligible && valid_eth_signature.verifies {
        res = build_messages_for_claim_and_whitelist_add(
            deps,
            info,
            eth_address,
            config.airdrop_amount,
        )?;
        claimed_amount = config.airdrop_amount;
    }
    Ok(res
        .add_attribute("claimed_amount", claimed_amount.to_string())
        .add_attribute("valid_eth_sig", valid_eth_signature.verifies.to_string())
        .add_attribute("eligible_at_request", is_eligible.to_string())
        .add_attribute("minter_address", config.minter_address.to_string()))
}

pub fn update_minter(
    deps: DepsMut,
    info: MessageInfo,
    minter_address: String,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {
            sender: info.sender,
        });
    }
    let minter_address = deps.api.addr_validate(&minter_address)?;
    config.minter_address = minter_address.clone();
    let _ = CONFIG.save(deps.storage, &config);
    let res = Response::new();
    Ok(res.add_attribute("updated_minter_address", minter_address.to_string()))
}

pub fn get_minter(deps: Deps) -> StdResult<Addr> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config.minter_address)
}
