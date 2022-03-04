#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::Order;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, StdResult};
use cw2::set_contract_version;
use cw_utils::Expiration;
use sg_std::fees::burn_and_distribute_fee;
use sg_std::{StargazeMsgWrapper, NATIVE_DENOM};

use crate::error::ContractError;
use crate::msg::{
    AddMembersMsg, ConfigResponse, ExecuteMsg, HasEndedResponse, HasMemberResponse,
    HasStartedResponse, InstantiateMsg, IsActiveResponse, MembersResponse, QueryMsg,
    RemoveMembersMsg,
};
use crate::state::{Config, CONFIG, WHITELIST};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg-whitelist";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// contract governance params
const MAX_MEMBERS: u32 = 5000;
const CREATION_FEE: u128 = 100_000_000;
const MIN_MINT_PRICE: u128 = 25_000_000;
const MAX_PER_ADDRESS_LIMIT: u32 = 30;

type Response = cosmwasm_std::Response<StargazeMsgWrapper>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mut msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    if msg.unit_price.denom != NATIVE_DENOM {
        return Err(ContractError::InvalidDenom(msg.unit_price.denom));
    }

    if msg.unit_price.amount.u128() < MIN_MINT_PRICE {
        return Err(ContractError::InvalidUnitPrice(
            msg.unit_price.amount.u128(),
            MIN_MINT_PRICE,
        ));
    }

    // Check per address limit is valid
    if msg.per_address_limit > MAX_PER_ADDRESS_LIMIT {
        return Err(ContractError::InvalidPerAddressLimit {
            max: MAX_PER_ADDRESS_LIMIT.to_string(),
            got: msg.per_address_limit.to_string(),
        });
    }
    if msg.per_address_limit == 0 {
        return Err(ContractError::InvalidPerAddressLimit {
            max: "must be > 0".to_string(),
            got: msg.per_address_limit.to_string(),
        });
    }

    // remove duplicate members
    msg.members.sort_unstable();
    msg.members.dedup();

    let config = Config {
        admin: info.sender.clone(),
        start_time: msg.start_time,
        end_time: msg.end_time,
        num_members: msg.members.len() as u32,
        unit_price: msg.unit_price,
        per_address_limit: msg.per_address_limit,
    };
    CONFIG.save(deps.storage, &config)?;

    if msg.start_time > msg.end_time {
        return Err(ContractError::InvalidStartTime(
            msg.start_time,
            msg.end_time,
        ));
    }

    if msg.start_time.is_expired(&env.block) {
        return Err(ContractError::InvalidStartTime(
            Expiration::AtTime(env.block.time),
            msg.start_time,
        ));
    }

    let fee_msgs = burn_and_distribute_fee(env, &info, CREATION_FEE)?;

    if MAX_MEMBERS <= (config.num_members) {
        return Err(ContractError::MembersExceeded {
            expected: MAX_MEMBERS,
            actual: config.num_members,
        });
    }

    for member in msg.members.into_iter() {
        let addr = deps.api.addr_validate(&member.clone())?;
        WHITELIST.save(deps.storage, addr, &true)?;
    }

    Ok(Response::new()
        .add_attribute("action", "instantiate")
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
        ExecuteMsg::UpdateStartTime(time) => execute_update_start_time(deps, env, info, time),
        ExecuteMsg::UpdateEndTime(time) => execute_update_end_time(deps, env, info, time),
        ExecuteMsg::AddMembers(msg) => execute_add_members(deps, env, info, msg),
        ExecuteMsg::RemoveMembers(msg) => execute_remove_members(deps, env, info, msg),
        ExecuteMsg::UpdatePerAddressLimit(per_address_limit) => {
            execute_update_per_address_limit(deps, env, info, per_address_limit)
        }
    }
}

pub fn execute_update_start_time(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    start_time: Expiration,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    // don't allow updating start time if whitelist is active
    if config.start_time.is_expired(&env.block) {
        return Err(ContractError::AlreadyStarted {});
    }

    if start_time > config.end_time {
        return Err(ContractError::InvalidStartTime(start_time, config.end_time));
    }

    config.start_time = start_time;
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "update_start_time")
        .add_attribute("start_time", start_time.to_string()))
}

pub fn execute_update_end_time(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    end_time: Expiration,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    // don't allow updating end time if whitelist is active
    if config.start_time.is_expired(&env.block) {
        return Err(ContractError::AlreadyStarted {});
    }

    if end_time > config.start_time {
        return Err(ContractError::InvalidEndTime(end_time, config.start_time));
    }

    config.end_time = end_time;
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "update_end_time")
        .add_attribute("end_time", end_time.to_string())
        .add_attribute("sender", info.sender))
}

pub fn execute_add_members(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: AddMembersMsg,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    for add in msg.to_add.into_iter() {
        if config.num_members >= MAX_MEMBERS {
            return Err(ContractError::MembersExceeded {
                expected: MAX_MEMBERS,
                actual: config.num_members,
            });
        }
        let addr = deps.api.addr_validate(&add)?;
        if WHITELIST.has(deps.storage, addr.clone()) {
            return Err(ContractError::DuplicateMember(addr.to_string()));
        }
        WHITELIST.save(deps.storage, addr, &true)?;
        config.num_members += 1;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "add_members")
        .add_attribute("sender", info.sender))
}

pub fn execute_remove_members(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: RemoveMembersMsg,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    for remove in msg.to_remove.into_iter() {
        let addr = deps.api.addr_validate(&remove)?;
        if !WHITELIST.has(deps.storage, addr.clone()) {
            return Err(ContractError::NoMemberFound(addr.to_string()));
        }
        WHITELIST.remove(deps.storage, addr);
        config.num_members -= 1;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "remove_members")
        .add_attribute("sender", info.sender))
}

pub fn execute_update_per_address_limit(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    per_address_limit: u32,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    if per_address_limit > MAX_PER_ADDRESS_LIMIT {
        return Err(ContractError::InvalidPerAddressLimit {
            max: MAX_PER_ADDRESS_LIMIT.to_string(),
            got: per_address_limit.to_string(),
        });
    }

    config.per_address_limit = per_address_limit;
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "update_per_address_limit")
        .add_attribute("per_address_limit", per_address_limit.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Members {} => to_binary(&query_members(deps)?),
        QueryMsg::HasStarted {} => to_binary(&query_has_started(deps, env)?),
        QueryMsg::HasEnded {} => to_binary(&query_has_ended(deps, env)?),
        QueryMsg::IsActive {} => to_binary(&query_is_active(deps, env)?),
        QueryMsg::HasMember { member } => to_binary(&query_has_member(deps, member)?),
        QueryMsg::Config {} => to_binary(&query_config(deps, env)?),
    }
}

fn query_has_started(deps: Deps, env: Env) -> StdResult<HasStartedResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(HasStartedResponse {
        has_started: config.start_time.is_expired(&env.block),
    })
}

fn query_has_ended(deps: Deps, env: Env) -> StdResult<HasEndedResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(HasEndedResponse {
        has_ended: config.end_time.is_expired(&env.block),
    })
}

fn query_is_active(deps: Deps, env: Env) -> StdResult<IsActiveResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(IsActiveResponse {
        is_active: config.start_time.is_expired(&env.block)
            && !config.end_time.is_expired(&env.block),
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

fn query_config(deps: Deps, env: Env) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        num_members: config.num_members,
        per_address_limit: config.per_address_limit,
        start_time: config.start_time,
        end_time: config.end_time,
        unit_price: config.unit_price,
        is_active: config.start_time.is_expired(&env.block)
            && !config.end_time.is_expired(&env.block),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{
        coin,
        testing::{mock_dependencies, mock_env, mock_info},
    };
    use sg_std::NATIVE_DENOM;

    const ADMIN: &str = "admin";
    const UNIT_AMOUNT: u128 = 100_000_000;

    const NON_EXPIRED_HEIGHT: Expiration = Expiration::AtHeight(22_222);
    const EXPIRED_HEIGHT: Expiration = Expiration::AtHeight(10);

    fn setup_contract(deps: DepsMut) {
        let msg = InstantiateMsg {
            members: vec!["adsfsa".to_string()],
            start_time: NON_EXPIRED_HEIGHT,
            end_time: NON_EXPIRED_HEIGHT,
            unit_price: coin(UNIT_AMOUNT, NATIVE_DENOM),
            per_address_limit: 1,
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
    fn improper_initialization() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            members: vec!["adsfsa".to_string()],
            start_time: NON_EXPIRED_HEIGHT,
            end_time: NON_EXPIRED_HEIGHT,
            unit_price: coin(1, NATIVE_DENOM),
            per_address_limit: 1,
        };
        let info = mock_info(ADMIN, &[coin(100_000_000, "ustars")]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    }

    #[test]
    fn improper_initialization_invalid_denom() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            members: vec!["adsfsa".to_string()],
            start_time: NON_EXPIRED_HEIGHT,
            end_time: NON_EXPIRED_HEIGHT,
            unit_price: coin(UNIT_AMOUNT, "not_ustars"),
            per_address_limit: 1,
        };
        let info = mock_info(ADMIN, &[coin(100_000_000, "ustars")]);
        let err = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(err.to_string(), "InvalidDenom: not_ustars");
    }

    #[test]
    fn improper_initialization_dedup() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            members: vec![
                "adsfsa".to_string(),
                "adsfsa".to_string(),
                "adsfsa".to_string(),
            ],
            start_time: NON_EXPIRED_HEIGHT,
            end_time: NON_EXPIRED_HEIGHT,
            unit_price: coin(UNIT_AMOUNT, NATIVE_DENOM),
            per_address_limit: 1,
        };
        let info = mock_info(ADMIN, &[coin(100_000_000, "ustars")]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        let res = query_config(deps.as_ref(), mock_env()).unwrap();
        assert_eq!(1, res.num_members);
    }

    #[test]
    fn check_start_time_after_end_time() {
        let msg = InstantiateMsg {
            members: vec!["adsfsa".to_string()],
            start_time: Expiration::AtHeight(101),
            end_time: Expiration::AtHeight(100),
            unit_price: coin(UNIT_AMOUNT, NATIVE_DENOM),
            per_address_limit: 1,
        };
        let info = mock_info(ADMIN, &[coin(100_000_000, "ustars")]);
        let mut deps = mock_dependencies();
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    }

    #[test]
    fn update_end_time() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());

        let msg = ExecuteMsg::UpdateEndTime(EXPIRED_HEIGHT);
        let info = mock_info(ADMIN, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.attributes.len(), 3);
        let res = query_config(deps.as_ref(), mock_env()).unwrap();
        assert_eq!(res.end_time, Expiration::AtHeight(10));
    }

    #[test]
    fn update_members() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());

        let add_msg = AddMembersMsg {
            to_add: vec!["adsfsa1".to_string()],
        };
        let msg = ExecuteMsg::AddMembers(add_msg);
        let info = mock_info(ADMIN, &[]);
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();
        assert_eq!(res.attributes.len(), 2);
        let res = query_members(deps.as_ref()).unwrap();
        assert_eq!(res.members.len(), 2);

        execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap_err();

        let remove_msg = RemoveMembersMsg {
            to_remove: vec!["adsfsa1".to_string()],
        };
        let msg = ExecuteMsg::RemoveMembers(remove_msg);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.attributes.len(), 2);
        let res = query_members(deps.as_ref()).unwrap();
        assert_eq!(res.members.len(), 1);
    }

    #[test]
    fn too_many_members_check() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());

        let mut members = vec![];
        for i in 0..MAX_MEMBERS {
            members.push(format!("adsfsa{}", i));
        }

        let inner_msg = AddMembersMsg { to_add: members };
        let msg = ExecuteMsg::AddMembers(inner_msg);
        let info = mock_info(ADMIN, &[]);
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(
            ContractError::MembersExceeded {
                expected: 5000,
                actual: 5000
            }
            .to_string(),
            err.to_string()
        );
    }

    #[test]
    fn update_per_address_limit() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());

        let per_address_limit: u32 = 50;
        let msg = ExecuteMsg::UpdatePerAddressLimit(per_address_limit);
        let info = mock_info(ADMIN, &[]);
        // let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        // let wl_config: ConfigResponse = query_config(deps.as_ref(), mock_env()).unwrap();
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(
            ContractError::InvalidPerAddressLimit {
                max: MAX_PER_ADDRESS_LIMIT.to_string(),
                got: per_address_limit.to_string(),
            }
            .to_string(),
            err.to_string()
        );

        let per_address_limit: u32 = 2;
        let msg = ExecuteMsg::UpdatePerAddressLimit(per_address_limit);
        let info = mock_info(ADMIN, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.attributes.len(), 2);
        let wl_config: ConfigResponse = query_config(deps.as_ref(), mock_env()).unwrap();
        assert_eq!(wl_config.per_address_limit, per_address_limit);
    }
}
