#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::Order;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;
use cw_utils::Expiration;
use sg_std::fees::burn_and_distribute_fee;
use sg_std::StargazeMsgWrapper;

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, HasEndedResponse, HasMemberResponse, InstantiateMsg, MembersResponse, QueryMsg,
    TimeResponse, UpdateMembersMsg,
};
use crate::state::{Config, CONFIG, WHITELIST};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg-whitelist";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// contract governance params
const MAX_MEMBERS: u32 = 1000;
const CREATION_FEE: u128 = 100_000_000;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response<StargazeMsgWrapper>, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let config = Config {
        admin: info.sender.clone(),
        end_time: msg.end_time,
        num_members: msg.members.len() as u32,
    };
    CONFIG.save(deps.storage, &config)?;

    let fee_msgs = burn_and_distribute_fee(env, &info, CREATION_FEE)?;

    if MAX_MEMBERS <= (config.num_members) {
        return Err(ContractError::MembersExceeded {
            expected: MAX_MEMBERS,
            actual: config.num_members,
        });
    }

    for member in msg.members.into_iter() {
        let addr = deps.api.addr_validate(&member.clone())?;
        WHITELIST.save(deps.storage, addr, &Empty {})?;
    }

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION)
        .add_attribute("sender", info.sender)
        .add_messages(fee_msgs))
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
    info: MessageInfo,
    end_time: Expiration,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    config.end_time = end_time;
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "update_end_time")
        .add_attribute("end_time", &end_time.to_string()))
}

pub fn execute_update_members(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: UpdateMembersMsg,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    for add in msg.add.into_iter() {
        if config.num_members >= MAX_MEMBERS {
            return Err(ContractError::MembersExceeded {
                expected: MAX_MEMBERS,
                actual: config.num_members,
            });
        }
        let addr = deps.api.addr_validate(&add)?;
        WHITELIST.save(deps.storage, addr, &Empty {})?;
        config.num_members += 1;
    }

    for remove in msg.remove.into_iter() {
        let addr = deps.api.addr_validate(&remove)?;
        WHITELIST.remove(deps.storage, addr);
        config.num_members -= 1;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "update_whitelist"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::EndTime {} => to_binary(&query_end_time(deps)?),
        QueryMsg::Members {} => to_binary(&query_members(deps)?),
        QueryMsg::HasEnded {} => to_binary(&query_has_ended(deps, env)?),
        QueryMsg::HasMember { member } => to_binary(&query_has_member(deps, member)?),
    }
}

fn query_end_time(deps: Deps) -> StdResult<TimeResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(TimeResponse {
        time: config.end_time.to_string(),
    })
}

fn query_has_ended(deps: Deps, env: Env) -> StdResult<HasEndedResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(HasEndedResponse {
        has_ended: config.end_time.is_expired(&env.block),
    })
}

fn query_members(deps: Deps) -> StdResult<MembersResponse> {
    let members = WHITELIST
        .range(deps.storage, None, None, Order::Ascending)
        .map(|addr| addr.unwrap().0.to_string())
        .collect::<Vec<String>>();

    Ok(MembersResponse { members })
}

fn query_has_member(deps: Deps, member: String) -> StdResult<HasMemberResponse> {
    let addr = deps.api.addr_validate(&member)?;

    Ok(HasMemberResponse {
        has_member: WHITELIST.has(deps.storage, addr),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{
        coin,
        testing::{mock_dependencies, mock_env, mock_info},
    };

    const ADMIN: &str = "admin";

    const NON_EXPIRED_HEIGHT: Expiration = Expiration::AtHeight(22_222);
    const EXPIRED_HEIGHT: Expiration = Expiration::AtHeight(10);

    fn setup_contract(deps: DepsMut) {
        let msg = InstantiateMsg {
            members: vec!["adsfsa".to_string()],
            end_time: NON_EXPIRED_HEIGHT,
        };
        let info = mock_info(ADMIN, &[coin(100_000_000, "ustars")]);
        let res = instantiate(deps, mock_env(), info, msg).unwrap();
        assert_eq!(2, res.messages.len());
    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
    }

    #[test]
    fn update_end_time() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());

        let msg = ExecuteMsg::UpdateEndTime(EXPIRED_HEIGHT);
        let info = mock_info(ADMIN, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.attributes.len(), 2);
        let res = query_end_time(deps.as_ref()).unwrap();
        assert_eq!(res.time, "expiration height: 10");
    }

    #[test]
    fn update_members() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());

        let inner_msg = UpdateMembersMsg {
            add: vec!["adsfsa1".to_string()],
            remove: vec![],
        };
        let msg = ExecuteMsg::UpdateMembers(inner_msg);
        let info = mock_info(ADMIN, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.attributes.len(), 1);
        let res = query_members(deps.as_ref()).unwrap();
        assert_eq!(res.members.len(), 2);
    }

    #[test]
    fn too_many_members_check() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());

        let inner_msg = UpdateMembersMsg {
            add: vec!["asdf".to_string(); MAX_MEMBERS as usize],
            remove: vec![],
        };
        let msg = ExecuteMsg::UpdateMembers(inner_msg);
        let info = mock_info(ADMIN, &[]);
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(
            ContractError::MembersExceeded {
                expected: 1000,
                actual: 1000
            }
            .to_string(),
            err.to_string()
        );
    }
}
