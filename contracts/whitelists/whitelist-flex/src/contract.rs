use crate::admin::{
    can_execute, execute_freeze, execute_update_admins, query_admin_list, query_can_execute,
};
use crate::error::ContractError;
use crate::helpers::validators::map_validate;
use crate::msg::{
    AddMembersMsg, ConfigResponse, ExecuteMsg, HasEndedResponse, HasMemberResponse,
    HasStartedResponse, InstantiateMsg, IsActiveResponse, Member, MembersResponse, QueryMsg,
    RemoveMembersMsg,
};
use crate::state::{AdminList, Config, ADMIN_LIST, CONFIG, WHITELIST};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cosmwasm_std::{Order, Timestamp};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use cw_utils::{may_pay, maybe_addr, must_pay};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use sg1::checked_fair_burn;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg-whitelist-flex";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// contract governance params
pub const MAX_MEMBERS: u32 = 5000;
pub const PRICE_PER_1000_MEMBERS: u128 = 100_000_000;
pub const MIN_MINT_PRICE: u128 = 0;

// queries
const PAGINATION_DEFAULT_LIMIT: u32 = 25;
const PAGINATION_MAX_LIMIT: u32 = 100;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    if msg.member_limit == 0 || msg.member_limit > MAX_MEMBERS {
        return Err(ContractError::InvalidMemberLimit {
            min: 1,
            max: MAX_MEMBERS,
            got: msg.member_limit,
        });
    }

    let creation_fee = Decimal::new(msg.member_limit.into(), 3)
        .ceil()
        .to_u128()
        .unwrap()
        * PRICE_PER_1000_MEMBERS;
    let payment = must_pay(&info, NATIVE_DENOM)?;
    if payment.u128() != creation_fee {
        return Err(ContractError::IncorrectCreationFee(
            payment.u128(),
            creation_fee,
        ));
    }

    if let Some(whale_cap) = msg.whale_cap {
        ensure!(
            whale_cap > msg.member_limit,
            ContractError::InvalidWhaleCap(whale_cap, msg.member_limit)
        );
    }

    let config = Config {
        start_time: msg.start_time,
        end_time: msg.end_time,
        num_members: msg.members.len() as u32,
        mint_price: msg.mint_price,
        member_limit: msg.member_limit,
        whale_cap: msg.whale_cap,
    };
    CONFIG.save(deps.storage, &config)?;

    let admin_config = AdminList {
        admins: map_validate(deps.api, &msg.admins)?,
        mutable: msg.admins_mutable,
    };
    ADMIN_LIST.save(deps.storage, &admin_config)?;

    if msg.start_time > msg.end_time {
        return Err(ContractError::InvalidStartTime(
            msg.start_time,
            msg.end_time,
        ));
    }

    if env.block.time >= msg.start_time {
        return Err(ContractError::InvalidStartTime(
            env.block.time,
            msg.start_time,
        ));
    }

    let genesis_start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    if msg.start_time < genesis_start_time {
        return Err(ContractError::InvalidStartTime(
            msg.start_time,
            genesis_start_time,
        ));
    }

    let mut res = Response::new();
    checked_fair_burn(&info, creation_fee, None, &mut res)?;

    if config.member_limit < config.num_members {
        return Err(ContractError::MembersExceeded {
            expected: config.member_limit,
            actual: config.num_members,
        });
    }

    for member in msg.members.into_iter() {
        let addr = deps.api.addr_validate(&member.address)?;
        if let Some(whale_cap) = config.whale_cap {
            if member.mint_count > whale_cap {
                return Err(ContractError::ExceededWhaleCap {});
            }
        }
        WHITELIST.save(deps.storage, addr, &member.mint_count)?;
    }

    Ok(res
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION)
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
        ExecuteMsg::AddMembers(msg) => execute_add_members(deps, env, info, msg),
        ExecuteMsg::RemoveMembers(msg) => execute_remove_members(deps, env, info, msg),
        ExecuteMsg::IncreaseMemberLimit(member_limit) => {
            execute_increase_member_limit(deps, info, member_limit)
        }
        ExecuteMsg::UpdateAdmins { admins } => execute_update_admins(deps, env, info, admins),
        ExecuteMsg::Freeze {} => execute_freeze(deps, env, info),
    }
}

pub fn execute_update_start_time(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    start_time: Timestamp,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    can_execute(&deps, info.sender.clone())?;

    // don't allow updating start time if whitelist is active
    if env.block.time >= config.start_time {
        return Err(ContractError::AlreadyStarted {});
    }

    if start_time > config.end_time {
        return Err(ContractError::InvalidStartTime(start_time, config.end_time));
    }

    let genesis_start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    let start_time = if start_time < genesis_start_time {
        genesis_start_time
    } else {
        start_time
    };

    config.start_time = start_time;
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "update_start_time")
        .add_attribute("start_time", start_time.to_string())
        .add_attribute("sender", info.sender))
}

pub fn execute_update_end_time(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    end_time: Timestamp,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    can_execute(&deps, info.sender.clone())?;

    // if whitelist already started don't allow updating end_time unless
    // it is to reduce it
    if env.block.time >= config.start_time && end_time > config.end_time {
        return Err(ContractError::AlreadyStarted {});
    }

    if end_time < config.start_time {
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
    can_execute(&deps, info.sender.clone())?;

    for add in msg.to_add.into_iter() {
        if config.num_members >= config.member_limit {
            return Err(ContractError::MembersExceeded {
                expected: config.member_limit,
                actual: config.num_members,
            });
        }
        let addr = deps.api.addr_validate(&add.address)?;
        if WHITELIST.has(deps.storage, addr.clone()) {
            return Err(ContractError::DuplicateMember(addr.to_string()));
        }
        WHITELIST.save(deps.storage, addr, &add.mint_count)?;
        config.num_members += 1;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "add_members")
        .add_attribute("sender", info.sender))
}

pub fn execute_remove_members(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: RemoveMembersMsg,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    can_execute(&deps, info.sender.clone())?;

    if env.block.time >= config.start_time {
        return Err(ContractError::AlreadyStarted {});
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

/// Increase member limit. Must include a fee if crossing 1000, 2000, etc member limit.
pub fn execute_increase_member_limit(
    deps: DepsMut,
    info: MessageInfo,
    member_limit: u32,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if config.member_limit >= member_limit || member_limit > MAX_MEMBERS {
        return Err(ContractError::InvalidMemberLimit {
            min: config.member_limit,
            max: MAX_MEMBERS,
            got: member_limit,
        });
    }

    // if new limit crosses 1,000 members, requires upgrade fee. Otherwise,  upgrade.
    let old_limit = Decimal::new(config.member_limit.into(), 3).ceil();
    let new_limit = Decimal::new(member_limit.into(), 3).ceil();
    let upgrade_fee: u128 = if new_limit > old_limit {
        (new_limit - old_limit).to_u128().unwrap() * PRICE_PER_1000_MEMBERS
    } else {
        0
    };
    let payment = may_pay(&info, NATIVE_DENOM)?;
    if payment.u128() != upgrade_fee {
        return Err(ContractError::IncorrectCreationFee(
            payment.u128(),
            upgrade_fee,
        ));
    }

    let mut res = Response::new();
    if upgrade_fee > 0 {
        checked_fair_burn(&info, upgrade_fee, None, &mut res)?
    }

    config.member_limit = member_limit;
    CONFIG.save(deps.storage, &config)?;
    Ok(res
        .add_attribute("action", "increase_member_limit")
        .add_attribute("member_limit", member_limit.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Members { start_after, limit } => {
            to_json_binary(&query_members(deps, start_after, limit)?)
        }

        QueryMsg::HasStarted {} => to_json_binary(&query_has_started(deps, env)?),
        QueryMsg::HasEnded {} => to_json_binary(&query_has_ended(deps, env)?),
        QueryMsg::IsActive {} => to_json_binary(&query_is_active(deps, env)?),
        QueryMsg::HasMember { member } => to_json_binary(&query_has_member(deps, member)?),
        QueryMsg::Member { member } => to_json_binary(&query_member(deps, member)?),
        QueryMsg::Config {} => to_json_binary(&query_config(deps, env)?),
        QueryMsg::AdminList {} => to_json_binary(&query_admin_list(deps)?),
        QueryMsg::CanExecute { sender, .. } => to_json_binary(&query_can_execute(deps, &sender)?),
    }
}

fn query_has_started(deps: Deps, env: Env) -> StdResult<HasStartedResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(HasStartedResponse {
        has_started: (env.block.time >= config.start_time),
    })
}

fn query_has_ended(deps: Deps, env: Env) -> StdResult<HasEndedResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(HasEndedResponse {
        has_ended: (env.block.time >= config.end_time),
    })
}

fn query_is_active(deps: Deps, env: Env) -> StdResult<IsActiveResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(IsActiveResponse {
        is_active: (env.block.time >= config.start_time) && (env.block.time < config.end_time),
    })
}

pub fn query_members(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<MembersResponse> {
    let limit = limit
        .unwrap_or(PAGINATION_DEFAULT_LIMIT)
        .min(PAGINATION_MAX_LIMIT) as usize;
    let start_addr = maybe_addr(deps.api, start_after)?;
    let start = start_addr.map(Bound::exclusive);
    let members = WHITELIST
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|res| {
            res.map(|(addr, mint_count)| Member {
                address: addr.into_string(),
                mint_count,
            })
        })
        .map(Result::unwrap)
        .collect::<Vec<_>>();

    Ok(MembersResponse { members })
}

pub fn query_has_member(deps: Deps, member: String) -> StdResult<HasMemberResponse> {
    let addr = deps.api.addr_validate(&member)?;

    Ok(HasMemberResponse {
        has_member: WHITELIST.has(deps.storage, addr),
    })
}

pub fn query_member(deps: Deps, member: String) -> StdResult<Member> {
    let addr = deps.api.addr_validate(&member)?;
    let mint_count = WHITELIST.load(deps.storage, addr.clone())?;
    Ok(Member {
        address: addr.into_string(),
        mint_count,
    })
}

pub fn query_config(deps: Deps, env: Env) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        num_members: config.num_members,
        member_limit: config.member_limit,
        start_time: config.start_time,
        end_time: config.end_time,
        mint_price: config.mint_price,
        is_active: (env.block.time >= config.start_time) && (env.block.time < config.end_time),
        whale_cap: config.whale_cap,
    })
}
