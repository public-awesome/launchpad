#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coins, to_binary, BankMsg, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult,
};
use cw2::set_contract_version;
use cw4::{Cw4Contract, Member, MemberListResponse, MemberResponse};
use sg_std::NATIVE_DENOM;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, SudoMsg};
use crate::state::{Config, Executor, CONFIG};

// version info for migration info
pub const CONTRACT_NAME: &str = "crates.io:sg-splits";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let group_addr = Cw4Contract(deps.api.addr_validate(&msg.group_addr).map_err(|_| {
        ContractError::InvalidGroup {
            addr: msg.group_addr.clone(),
        }
    })?);
    let weight = group_addr.total_weight(&deps.querier)?;
    if weight == 0 {
        return Err(ContractError::InvalidWeight { weight });
    }

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    if let Some(Executor::Only(ref addr)) = msg.executor {
        deps.api.addr_validate(addr.as_ref())?;
    }

    let cfg = Config {
        group_addr,
        executor: msg.executor,
    };
    CONFIG.save(deps.storage, &cfg)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Distribute {} => execute_distribute(deps, env, info),
        ExecuteMsg::UpdateExecutor { executor } => execute_update_executor(deps, info, executor),
    }
}

pub fn execute_update_executor(
    deps: DepsMut,
    info: MessageInfo,
    executor: Option<Executor>,
) -> Result<Response, ContractError> {
    if let Some(Executor::Only(ref addr)) = executor {
        if addr != &info.sender {
            return Err(ContractError::Unauthorized {});
        }

        deps.api.addr_validate(addr.as_ref())?;

        let mut cfg = CONFIG.load(deps.storage)?;
        cfg.executor = executor;
        CONFIG.save(deps.storage, &cfg)?;
    } else {
        return Err(ContractError::Unauthorized {});
    }

    Ok(Response::default())
}

pub fn execute_distribute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    config.authorize(&deps.querier, &info.sender)?;

    let total_weight = config.group_addr.total_weight(&deps.querier)?;
    if total_weight == 0 {
        return Err(ContractError::InvalidWeight {
            weight: total_weight,
        });
    }

    let members = config.group_addr.list_members(&deps.querier, None, None)?;

    let funds = deps
        .querier
        .query_balance(env.contract.address, NATIVE_DENOM)?;
    if funds.amount.is_zero() {
        return Err(ContractError::NoFunds {});
    }

    let msgs = members
        .iter()
        .map(|member| {
            let ratio = Decimal::from_ratio(member.weight, total_weight);
            let amount = funds.amount * ratio;
            BankMsg::Send {
                to_address: member.addr.clone(),
                amount: coins(amount.u128(), funds.denom.clone()),
            }
        })
        .collect::<Vec<_>>();

    Ok(Response::new()
        .add_attribute("action", "distribute")
        .add_messages(msgs))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, _env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    match msg {
        SudoMsg::UpdateExecutor { executor } => {
            if let Some(Executor::Only(ref addr)) = executor {
                deps.api.addr_validate(addr.as_ref())?;
            }

            let mut cfg = CONFIG.load(deps.storage)?;
            cfg.executor = executor;
            CONFIG.save(deps.storage, &cfg)?;
        }
    }

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::ListMembers { start_after, limit } => {
            to_binary(&list_members(deps, start_after, limit)?)
        }
        QueryMsg::Member { address } => to_binary(&query_member(deps, address)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    Ok(ConfigResponse { config })
}

fn query_member(deps: Deps, member: String) -> StdResult<MemberResponse> {
    let cfg = CONFIG.load(deps.storage)?;
    let voter_addr = deps.api.addr_validate(&member)?;
    let weight = cfg.group_addr.is_member(&deps.querier, &voter_addr, None)?;

    Ok(MemberResponse { weight })
}

fn list_members(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<MemberListResponse> {
    let cfg = CONFIG.load(deps.storage)?;
    let members = cfg
        .group_addr
        .list_members(&deps.querier, start_after, limit)?
        .into_iter()
        .map(|member| Member {
            addr: member.addr,
            weight: member.weight,
        })
        .collect();
    Ok(MemberListResponse { members })
}
