#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use cw4_group::contract::{create, execute_update_members};
use cw_utils::Expiration;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, TimeResponse};
use crate::state::{Config, CONFIG};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg-whitelist";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let config = Config {
        start_time: msg.start_time,
        end_time: msg.end_time,
    };
    CONFIG.save(deps.storage, &config)?;

    create(deps, msg.minter, msg.members, env.block.height)?;

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
        ExecuteMsg::UpdateStartTime(time) => execute_update_start_time(deps, env, info, time),
        ExecuteMsg::UpdateEndTime(time) => execute_update_end_time(deps, env, info, time),
        ExecuteMsg::UpdateMembers { remove, add } => {
            execute_update_members(deps, env, info, add, remove)
                .map_err(ContractError::C4ContractError)
        }
    }
}

pub fn execute_update_start_time(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    start_time: Expiration,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    // if info.sender != config.minter {
    //     return Err(ContractError::Unauthorized {});
    // }
    config.start_time = start_time;
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attribute("action", "update_start_time"))
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::StartTime {} => to_binary(&query_start_time(deps)?),
        QueryMsg::EndTime {} => to_binary(&query_end_time(deps)?),
        // QueryMsg::WhitelistAddresses {} => to_binary(&query_whitelist_addresses(deps)?),
        // QueryMsg::WhitelistExpiration {} => to_binary(&query_whitelist_expiration(deps)?),
        // QueryMsg::OnWhitelist { address } => to_binary(&query_on_whitelist(deps, address)?),
    }
}

fn query_start_time(deps: Deps) -> StdResult<TimeResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(TimeResponse {
        time: config.start_time.to_string(),
    })
}

fn query_end_time(deps: Deps) -> StdResult<TimeResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(TimeResponse {
        time: config.end_time.to_string(),
    })
}
