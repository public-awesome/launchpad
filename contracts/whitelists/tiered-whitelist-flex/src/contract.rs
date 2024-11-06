use crate::admin::{
    can_execute, execute_freeze, execute_update_admins, query_admin_list, query_can_execute,
};
use crate::error::ContractError;
use crate::helpers::validators::map_validate;
use crate::helpers::{fetch_active_stage, fetch_active_stage_index, validate_stages};
use crate::msg::{
    AddMembersMsg, AllStageMemberInfoResponse, ConfigResponse, ExecuteMsg, HasEndedResponse,
    HasMemberResponse, HasStartedResponse, InstantiateMsg, IsActiveResponse, Member,
    MembersResponse, QueryMsg, RemoveMembersMsg, StageMemberInfoResponse, StageResponse,
    StagesResponse, UpdateStageConfigMsg,
};
use crate::state::{AdminList, Config, Stage, ADMIN_LIST, CONFIG, MEMBER_COUNT, WHITELIST_STAGES};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure, to_json_binary, Binary, Coin, Deps, DepsMut, Env, MessageInfo, StdResult, Timestamp,
    Uint128,
};
use cosmwasm_std::{Order, StdError};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use cw_utils::{may_pay, maybe_addr, must_pay};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use sg1::checked_fair_burn;
use sg_std::{Response, NATIVE_DENOM};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg-tiered-whitelist-flex";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// contract governance params
pub const MAX_MEMBERS: u32 = 30000;
pub const PRICE_PER_1000_MEMBERS: u128 = 100_000_000;
pub const MIN_MINT_PRICE: u128 = 0;
pub const MAX_PER_ADDRESS_LIMIT: u32 = 30;

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

    validate_stages(&env, &msg.stages)?;

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
        stages: msg.stages.clone(),
        num_members: msg.members.iter().map(|m| m.len() as u32).sum(),
        member_limit: msg.member_limit,
        whale_cap: msg.whale_cap,
    };
    CONFIG.save(deps.storage, &config)?;

    let admin_config = AdminList {
        admins: map_validate(deps.api, &msg.admins)?,
        mutable: msg.admins_mutable,
    };
    ADMIN_LIST.save(deps.storage, &admin_config)?;

    let mut res = Response::new();
    checked_fair_burn(&info, creation_fee, None, &mut res)?;

    if config.member_limit < config.num_members {
        return Err(ContractError::MembersExceeded {
            expected: config.member_limit,
            actual: config.num_members,
        });
    }

    for stage in 0..msg.stages.clone().len() {
        MEMBER_COUNT.save(
            deps.storage,
            stage as u32,
            &(msg.members[stage].len() as u32),
        )?;
        for member in msg.members[stage].iter() {
            let addr = deps.api.addr_validate(&member.address)?;
            if let Some(whale_cap) = config.whale_cap {
                if member.mint_count > whale_cap {
                    return Err(ContractError::ExceededWhaleCap {});
                }
            }
            WHITELIST_STAGES.save(deps.storage, (stage as u32, addr), &member.mint_count)?;
        }
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
        ExecuteMsg::UpdateStageConfig(msg) => execute_update_stage_config(deps, env, info, msg),
        ExecuteMsg::AddStage(msg) => execute_add_stage(deps, env, info, msg.stage, msg.members),
        ExecuteMsg::RemoveStage(msg) => execute_remove_stage(deps, env, info, msg.stage_id),
        ExecuteMsg::AddMembers(msg) => execute_add_members(deps, env, info, msg),
        ExecuteMsg::RemoveMembers(msg) => execute_remove_members(deps, env, info, msg),
        ExecuteMsg::IncreaseMemberLimit(member_limit) => {
            execute_increase_member_limit(deps, info, member_limit)
        }
        ExecuteMsg::UpdateAdmins { admins } => execute_update_admins(deps, env, info, admins),
        ExecuteMsg::Freeze {} => execute_freeze(deps, env, info),
    }
}

pub fn execute_update_stage_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: UpdateStageConfigMsg,
) -> Result<Response, ContractError> {
    can_execute(&deps, info.sender.clone())?;
    let mut config = CONFIG.load(deps.storage)?;
    let stage_id = msg.stage_id as usize;
    let updated_stage = Stage {
        name: msg.name.unwrap_or(config.stages[stage_id].clone().name),
        start_time: msg
            .start_time
            .unwrap_or(config.stages[stage_id].clone().start_time),
        end_time: msg
            .end_time
            .unwrap_or(config.stages[stage_id].clone().end_time),
        mint_price: msg
            .mint_price
            .unwrap_or(config.stages[stage_id].clone().mint_price),
    };
    config.stages[stage_id] = updated_stage.clone();
    validate_stages(&env, &config.stages)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_stage_config")
        .add_attribute("stage_id", stage_id.to_string())
        .add_attribute("name", updated_stage.clone().name)
        .add_attribute("start_time", updated_stage.clone().start_time.to_string())
        .add_attribute("end_time", updated_stage.clone().end_time.to_string())
        .add_attribute("mint_price", updated_stage.clone().mint_price.to_string())
        .add_attribute("sender", info.sender))
}

pub fn execute_add_members(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: AddMembersMsg,
) -> Result<Response, ContractError> {
    can_execute(&deps, info.sender.clone())?;
    let mut config = CONFIG.load(deps.storage)?;
    ensure!(
        msg.stage_id < config.stages.len() as u32,
        ContractError::StageNotFound {}
    );

    let mut members_added = 0;
    for add in msg.to_add.into_iter() {
        if config.num_members >= config.member_limit {
            return Err(ContractError::MembersExceeded {
                expected: config.member_limit,
                actual: config.num_members,
            });
        }
        let addr = deps.api.addr_validate(&add.address)?;
        if WHITELIST_STAGES.has(deps.storage, (msg.stage_id, addr.clone())) {
            continue;
        }
        members_added += 1;
        WHITELIST_STAGES.save(deps.storage, (msg.stage_id, addr.clone()), &add.mint_count)?;
        MEMBER_COUNT.update(deps.storage, msg.stage_id, |count| {
            Ok::<u32, StdError>(count.unwrap_or(0) + 1)
        })?;
        config.num_members += 1;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "add_members")
        .add_attribute("stage_id", msg.stage_id.to_string())
        .add_attribute("num_members", config.num_members.to_string())
        .add_attribute("members_added", members_added.to_string())
        .add_attribute("sender", info.sender))
}

pub fn execute_remove_members(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: RemoveMembersMsg,
) -> Result<Response, ContractError> {
    can_execute(&deps, info.sender.clone())?;
    let mut config = CONFIG.load(deps.storage)?;
    ensure!(
        msg.stage_id < config.stages.len() as u32,
        ContractError::StageNotFound {}
    );

    ensure!(
        env.block.time < config.stages[msg.stage_id as usize].start_time,
        ContractError::AlreadyStarted {}
    );

    for remove in msg.to_remove.into_iter() {
        let addr = deps.api.addr_validate(&remove)?;
        if !WHITELIST_STAGES.has(deps.storage, (msg.stage_id, addr.clone())) {
            return Err(ContractError::NoMemberFound(addr.to_string()));
        }
        WHITELIST_STAGES.remove(deps.storage, (msg.stage_id, addr.clone()));
        MEMBER_COUNT.update(deps.storage, msg.stage_id, |count| {
            Ok::<u32, StdError>(count.unwrap_or(0).saturating_sub(1))
        })?;
        config.num_members -= 1;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "remove_members")
        .add_attribute("stage_id", msg.stage_id.to_string())
        .add_attribute("sender", info.sender))
}

pub fn execute_add_stage(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: Stage,
    members: Vec<Member>,
) -> Result<Response, ContractError> {
    can_execute(&deps, info.sender.clone())?;
    let mut config = CONFIG.load(deps.storage)?;
    ensure!(
        config.stages.len().lt(&3),
        ContractError::MaxStageCountExceeded {}
    );
    config.stages.push(msg.clone());
    validate_stages(&env, &config.stages)?;
    let stage_id = config.stages.len().saturating_sub(1) as u32;

    for add in members.clone().into_iter() {
        if config.num_members >= config.member_limit {
            return Err(ContractError::MembersExceeded {
                expected: config.member_limit,
                actual: config.num_members,
            });
        }
        let addr = deps.api.addr_validate(&add.address)?;
        if let Some(whale_cap) = config.whale_cap {
            if add.mint_count > whale_cap {
                return Err(ContractError::ExceededWhaleCap {});
            }
        }
        if WHITELIST_STAGES.has(deps.storage, (stage_id, addr.clone())) {
            continue;
        }
        WHITELIST_STAGES.save(deps.storage, (stage_id, addr.clone()), &add.mint_count)?;
        config.num_members += 1;
    }
    MEMBER_COUNT.save(deps.storage, stage_id, &(members.len() as u32))?;

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "add_stage")
        .add_attribute("stage_id", config.stages.len().to_string())
        .add_attribute("sender", info.sender))
}

pub fn execute_remove_stage(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    stage_id: u32,
) -> Result<Response, ContractError> {
    can_execute(&deps, info.sender.clone())?;
    let mut config = CONFIG.load(deps.storage)?;
    ensure!(
        config.stages.len().gt(&(stage_id as usize)),
        ContractError::StageNotFound {}
    );

    ensure!(
        env.block.time < config.stages[stage_id as usize].start_time,
        ContractError::AlreadyStarted {}
    );
    // remove the stage and following stages permanently
    config.stages = config.stages.into_iter().take(stage_id as usize).collect();

    // remove members from the WHITELIST_STAGES for stage_id and following stages. Reduce the num_members count
    for stage in stage_id..config.stages.len() as u32 {
        let members = WHITELIST_STAGES
            .prefix(stage)
            .keys(deps.storage, None, None, Order::Ascending)
            .map(|key| key.unwrap())
            .collect::<Vec<_>>();
        for member in members.into_iter() {
            WHITELIST_STAGES.remove(deps.storage, (stage, member));
            config.num_members -= 1;
        }
        MEMBER_COUNT.remove(deps.storage, stage);
    }

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "add_stage")
        .add_attribute("stage_id", config.stages.len().to_string())
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
        QueryMsg::Members {
            start_after,
            limit,
            stage_id,
        } => to_json_binary(&query_members(deps, start_after, limit, stage_id)?),

        QueryMsg::HasStarted {} => to_json_binary(&query_has_started(deps, env)?),
        QueryMsg::HasEnded {} => to_json_binary(&query_has_ended(deps, env)?),
        QueryMsg::IsActive {} => to_json_binary(&query_is_active(deps, env)?),
        QueryMsg::ActiveStage {} => to_json_binary(&fetch_active_stage(deps.storage, &env)),
        QueryMsg::ActiveStageId {} => {
            to_json_binary(&fetch_active_stage_index(deps.storage, &env).map_or(0, |i| i + 1))
        }
        QueryMsg::HasMember { member } => to_json_binary(&query_has_member(deps, env, member)?),
        QueryMsg::StageMemberInfo { stage_id, member } => {
            to_json_binary(&query_stage_member_info(deps, stage_id, member)?)
        }
        QueryMsg::AllStageMemberInfo { member } => {
            to_json_binary(&query_all_stage_member_info(deps, member)?)
        }
        QueryMsg::Config {} => to_json_binary(&query_config(deps, env)?),
        QueryMsg::Stage { stage_id } => to_json_binary(&query_stage(deps, stage_id)?),
        QueryMsg::Stages {} => to_json_binary(&query_stage_list(deps)?),
        QueryMsg::AdminList {} => to_json_binary(&query_admin_list(deps)?),
        QueryMsg::CanExecute { sender, .. } => to_json_binary(&query_can_execute(deps, &sender)?),
        QueryMsg::Member { member } => to_json_binary(&query_member(deps, env, member)?),
    }
}

fn query_has_started(deps: Deps, env: Env) -> StdResult<HasStartedResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(HasStartedResponse {
        has_started: !config.stages.is_empty() && (env.block.time >= config.stages[0].start_time),
    })
}

fn query_has_ended(deps: Deps, env: Env) -> StdResult<HasEndedResponse> {
    let config = CONFIG.load(deps.storage)?;
    let stage_count = config.stages.len();
    Ok(HasEndedResponse {
        has_ended: (stage_count > 0) && (env.block.time >= config.stages[stage_count - 1].end_time),
    })
}

fn query_is_active(deps: Deps, env: Env) -> StdResult<IsActiveResponse> {
    Ok(IsActiveResponse {
        is_active: fetch_active_stage(deps.storage, &env).is_some(),
    })
}

pub fn query_members(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
    stage_id: u32,
) -> StdResult<MembersResponse> {
    let limit = limit
        .unwrap_or(PAGINATION_DEFAULT_LIMIT)
        .min(PAGINATION_MAX_LIMIT) as usize;
    let start_addr = maybe_addr(deps.api, start_after)?;
    let start = start_addr.map(Bound::exclusive);
    let members = WHITELIST_STAGES
        .prefix(stage_id)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|addr| {
            let (k, v) = addr?;
            Ok(Member {
                address: k.to_string(),
                mint_count: v,
            })
        })
        .collect::<StdResult<Vec<Member>>>()?;

    Ok(MembersResponse { members })
}

pub fn query_has_member(deps: Deps, env: Env, member: String) -> StdResult<HasMemberResponse> {
    let addr = deps.api.addr_validate(&member)?;
    let active_stage_id = fetch_active_stage_index(deps.storage, &env);
    let has_member = match active_stage_id {
        Some(stage_id) => WHITELIST_STAGES.has(deps.storage, (stage_id, addr)),
        None => false,
    };
    Ok(HasMemberResponse { has_member })
}

pub fn query_stage_member_info(
    deps: Deps,
    stage_id: u32,
    member: String,
) -> StdResult<StageMemberInfoResponse> {
    let addr = deps.api.addr_validate(&member)?;
    let mint_count = WHITELIST_STAGES.may_load(deps.storage, (stage_id, addr.clone()))?;
    Ok(StageMemberInfoResponse {
        stage_id,
        is_member: mint_count.is_some(),
        per_address_limit: mint_count.unwrap_or(0),
    })
}

pub fn query_all_stage_member_info(
    deps: Deps,
    member: String,
) -> StdResult<AllStageMemberInfoResponse> {
    let addr = deps.api.addr_validate(&member)?;
    let config = CONFIG.load(deps.storage)?;
    let mut all_stage_member_info = vec![];
    for stage_id in 0..config.stages.len() {
        let mint_count =
            WHITELIST_STAGES.may_load(deps.storage, (stage_id as u32, addr.clone()))?;
        all_stage_member_info.push(StageMemberInfoResponse {
            stage_id: stage_id as u32,
            is_member: mint_count.is_some(),
            per_address_limit: mint_count.unwrap_or(0),
        });
    }
    Ok(AllStageMemberInfoResponse {
        all_stage_member_info,
    })
}

pub fn query_member(deps: Deps, env: Env, member: String) -> StdResult<Member> {
    let addr = deps.api.addr_validate(&member)?;
    let active_stage_id = fetch_active_stage_index(deps.storage, &env);
    if active_stage_id.is_none() {
        return Err(StdError::generic_err("No active stage found"));
    }
    let mint_count =
        WHITELIST_STAGES.load(deps.storage, (active_stage_id.unwrap(), addr.clone()))?;
    Ok(Member {
        address: addr.into_string(),
        mint_count,
    })
}

pub fn query_config(deps: Deps, env: Env) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    let active_stage = fetch_active_stage(deps.storage, &env);
    if let Some(stage) = active_stage {
        Ok(ConfigResponse {
            num_members: config.num_members,
            member_limit: config.member_limit,
            start_time: stage.start_time,
            end_time: stage.end_time,
            mint_price: stage.mint_price,
            whale_cap: config.whale_cap,
            is_active: true,
        })
    } else if !config.stages.is_empty() {
        let stage = if env.block.time < config.stages[0].start_time {
            config.stages[0].clone()
        } else {
            config.stages[config.stages.len() - 1].clone()
        };
        Ok(ConfigResponse {
            num_members: config.num_members,
            member_limit: config.member_limit,
            start_time: stage.start_time,
            end_time: stage.end_time,
            mint_price: stage.mint_price,
            whale_cap: config.whale_cap,
            is_active: false,
        })
    } else {
        Ok(ConfigResponse {
            num_members: config.num_members,
            member_limit: config.member_limit,
            start_time: Timestamp::from_seconds(0),
            end_time: Timestamp::from_seconds(0),
            mint_price: Coin {
                denom: NATIVE_DENOM.to_string(),
                amount: Uint128::zero(),
            },
            whale_cap: config.whale_cap,
            is_active: false,
        })
    }
}

pub fn query_stage(deps: Deps, stage_id: u32) -> StdResult<StageResponse> {
    let config = CONFIG.load(deps.storage)?;
    ensure!(
        stage_id < config.stages.len() as u32,
        StdError::generic_err("Stage not found")
    );
    Ok(StageResponse {
        stage_id,
        stage: config.stages[stage_id as usize].clone(),
        member_count: MEMBER_COUNT.may_load(deps.storage, stage_id)?.unwrap_or(0),
    })
}

pub fn query_stage_list(deps: Deps) -> StdResult<StagesResponse> {
    let config = CONFIG.load(deps.storage)?;
    ensure!(
        !config.stages.is_empty(),
        StdError::generic_err("No stages found")
    );
    let stages = config
        .stages
        .iter()
        .enumerate()
        .map(|(i, stage)| StageResponse {
            stage_id: i as u32,
            stage: stage.clone(),
            member_count: MEMBER_COUNT
                .may_load(deps.storage, i as u32)
                .unwrap_or(Some(0u32))
                .unwrap_or(0u32),
        })
        .collect();
    Ok(StagesResponse { stages })
}
