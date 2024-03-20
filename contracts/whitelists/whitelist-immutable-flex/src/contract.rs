use crate::state::{Config, CONFIG, TOTAL_ADDRESS_COUNT, WHITELIST};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, Member, MemberResponse, QueryMsg};
use cw_utils::nonpayable;
use sg_std::Response;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:whitelist-immutable-flex";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    mut msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let config = Config {
        admin: info.sender,
        mint_discount_bps: msg.mint_discount_bps,
    };

    msg.members.dedup();
    let count = update_whitelist(&mut deps, msg)?;
    validate_nonempty_whitelist(count)?;
    TOTAL_ADDRESS_COUNT.save(deps.storage, &count)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default()
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION))
}

fn update_whitelist(deps: &mut DepsMut, msg: InstantiateMsg) -> Result<u64, ContractError> {
    let mut count = 0u64;
    for member in msg.members.into_iter() {
        WHITELIST.save(deps.storage, &member.address, &member.mint_count)?;
        count += 1;
    }
    Ok(count)
}

fn validate_nonempty_whitelist(count: u64) -> Result<bool, ContractError> {
    if count < 1 {
        return Err(ContractError::EmptyWhitelist {});
    }
    Ok(true)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::HasMember { address } => to_json_binary(&query_has_member(deps, address)?),
        QueryMsg::Admin {} => to_json_binary(&query_admin(deps)?),
        QueryMsg::AddressCount {} => to_json_binary(&query_address_count(deps)?),
        QueryMsg::Member { address } => to_json_binary(&query_member(deps, address)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse { config })
}

pub fn query_has_member(deps: Deps, address: String) -> StdResult<bool> {
    Ok(WHITELIST.has(deps.storage, &address))
}

pub fn query_admin(deps: Deps) -> StdResult<String> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config.admin.to_string())
}

pub fn query_address_count(deps: Deps) -> StdResult<u64> {
    TOTAL_ADDRESS_COUNT.load(deps.storage)
}

pub fn query_member(deps: Deps, address: String) -> StdResult<MemberResponse> {
    let mint_count = WHITELIST.load(deps.storage, &address)?;
    Ok(MemberResponse {
        member: Member {
            address,
            mint_count,
        },
    })
}
