#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::Order;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;
use cw_utils::Expiration;

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, IsValidResponse, MembersResponse, QueryMsg, TimeResponse,
    UpdateMembersMsg,
};
use crate::state::{Config, CONFIG, WHITELIST};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg-whitelist";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// contract governance params
const MAX_WHITELIST_ADDRS_LENGTH: u32 = 10000;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let config = Config {
        end_time: msg.end_time,
        num_addresses: msg.members.len() as u32,
    };
    CONFIG.save(deps.storage, &config)?;

    if MAX_WHITELIST_ADDRS_LENGTH <= (config.num_addresses) {
        return Err(ContractError::MaxWhitelistAddressLengthExceeded {});
    }

    for member in msg.members.clone().into_iter() {
        let addr = deps.api.addr_validate(&member.clone())?;
        WHITELIST.save(deps.storage, addr, &Empty {})?;
    }

    Ok(Response::new()
        .add_attribute("method", "instantiate_whitelist")
        .add_attribute("sender", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateEndTime(time) => execute_update_end_time(deps, env, info, time),
        ExecuteMsg::UpdateMembers(msg) => execute_update_members(deps, env, info, msg),
    }
}

pub fn execute_update_end_time(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    end_time: Expiration,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    // if info.sender != config.admin {
    //     return Err(ContractError::Unauthorized {});
    // }

    config.end_time = end_time;
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attribute("action", "update_end_time"))
}

pub fn execute_update_members(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: UpdateMembersMsg,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    // if info.sender != config.admin {
    //     return Err(ContractError::Unauthorized {});
    // }

    for add in msg.add.into_iter() {
        let addr = deps.api.addr_validate(&add)?;
        WHITELIST.save(deps.storage, addr, &Empty {})?;
        config.num_addresses += 1;
    }

    for remove in msg.remove.into_iter() {
        let addr = deps.api.addr_validate(&remove)?;
        WHITELIST.remove(deps.storage, addr);
        config.num_addresses -= 1;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "update_whitelist"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::EndTime {} => to_binary(&query_end_time(deps)?),
        QueryMsg::Members {} => to_binary(&query_members(deps)?),
        QueryMsg::IsValidMember { member } => to_binary(&query_is_valid_member(deps, env, member)?),
    }
}

fn query_end_time(deps: Deps) -> StdResult<TimeResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(TimeResponse {
        time: config.end_time.to_string(),
    })
}

fn query_members(deps: Deps) -> StdResult<MembersResponse> {
    let members = WHITELIST
        .range(deps.storage, None, None, Order::Ascending)
        .map(|addr| addr.unwrap().0.to_string())
        .collect::<Vec<String>>();

    Ok(MembersResponse { members })
}

fn query_is_valid_member(deps: Deps, env: Env, member: String) -> StdResult<IsValidResponse> {
    let config = CONFIG.load(deps.storage)?;
    let addr = deps.api.addr_validate(&member)?;

    let allowlist = WHITELIST.has(deps.storage, addr);
    let is_valid = allowlist && !config.end_time.is_expired(&env.block);

    Ok(IsValidResponse { is_valid })
}
