#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coins, to_json_binary, Addr, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdResult, SubMsg, Uint128,
};
use cw2::set_contract_version;
use cw4::{Cw4Contract, Member, MemberListResponse, MemberResponse};
use cw_utils::{maybe_addr, parse_reply_instantiate_data};
use sg_std::NATIVE_DENOM;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, Group, InstantiateMsg, QueryMsg};
use crate::state::{ADMIN, GROUP};

// Version info for migration info
pub const CONTRACT_NAME: &str = "crates.io:sg-splits";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const INIT_GROUP_REPLY_ID: u64 = 1;

// This is the same hardcoded value as in cw4-group
pub const PAGINATION_LIMIT: u32 = 30;
// We hardcode a smaller number to effectively check group size
pub const MAX_GROUP_SIZE: u32 = 25;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let self_addr = env.contract.address;

    let admin_addr = maybe_addr(deps.api, msg.admin)?;
    ADMIN.set(deps.branch(), admin_addr)?;

    match msg.group {
        Group::Cw4Instantiate(init) => Ok(Response::default().add_submessage(
            SubMsg::reply_on_success(init.into_wasm_msg(self_addr), INIT_GROUP_REPLY_ID),
        )),
        Group::Cw4Address(addr) => {
            let group = Cw4Contract(
                deps.api
                    .addr_validate(&addr)
                    .map_err(|_| ContractError::InvalidGroup { addr })?,
            );

            checked_total_weight(&group, deps.as_ref())?;
            checked_total_members(&group, deps.as_ref())?;

            GROUP.save(deps.storage, &group)?;
            Ok(Response::default())
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let api = deps.api;

    match msg {
        ExecuteMsg::UpdateAdmin { admin } => {
            Ok(ADMIN.execute_update_admin(deps, info, maybe_addr(api, admin)?)?)
        }
        ExecuteMsg::Distribute {} => execute_distribute(deps.as_ref(), env, info),
    }
}

pub fn execute_distribute(
    deps: Deps,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    if !can_distribute(deps, info)? {
        return Err(ContractError::Unauthorized {});
    }

    let group = GROUP.load(deps.storage)?;

    let total_weight = checked_total_weight(&group, deps)?;
    let members = group.list_members(&deps.querier, None, Some(PAGINATION_LIMIT))?;
    let members_count = members.len();
    if members_count == 0 || members_count > MAX_GROUP_SIZE as usize {
        return Err(ContractError::InvalidMemberCount {
            count: members_count,
        });
    }

    let funds = deps
        .querier
        .query_balance(env.contract.address, NATIVE_DENOM)?;
    if funds.amount.is_zero() {
        return Err(ContractError::NoFunds {});
    }

    // To avoid rounding errors, distribute funds modulo the total weight.
    // Keep remaining balance in the contract.
    let multiplier = funds.amount / Uint128::from(total_weight);
    if multiplier.is_zero() {
        return Err(ContractError::NotEnoughFunds { min: total_weight });
    }

    let msgs = members
        .iter()
        .filter(|m| m.weight > 0)
        .map(|member| {
            let amount = multiplier * Uint128::from(member.weight);
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

fn checked_total_weight(group: &Cw4Contract, deps: Deps) -> Result<u64, ContractError> {
    let weight = group.total_weight(&deps.querier)?;
    if weight == 0 {
        return Err(ContractError::InvalidWeight { weight });
    }

    Ok(weight)
}

fn checked_total_members(group: &Cw4Contract, deps: Deps) -> Result<u64, ContractError> {
    let members = group
        .list_members(&deps.querier, None, Some(PAGINATION_LIMIT))?
        .len();
    if members == 0 || members > MAX_GROUP_SIZE as usize {
        return Err(ContractError::InvalidMemberCount { count: members });
    }

    Ok(members as u64)
}

/// Checks if the sender is an admin or a member of a group.
fn can_distribute(deps: Deps, info: MessageInfo) -> StdResult<bool> {
    match ADMIN.get(deps)? {
        Some(admin) => Ok(admin == info.sender),
        None => Ok(GROUP
            .load(deps.storage)?
            .is_member(&deps.querier, &info.sender, None)?
            .is_some()),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Admin {} => to_json_binary(&ADMIN.query_admin(deps)?),
        QueryMsg::Group {} => to_json_binary(&query_group(deps)?),
        QueryMsg::ListMembers { start_after, limit } => {
            to_json_binary(&list_members(deps, start_after, limit)?)
        }
        QueryMsg::Member { address } => to_json_binary(&query_member(deps, address)?),
    }
}

fn query_group(deps: Deps) -> StdResult<Addr> {
    Ok(GROUP.load(deps.storage)?.addr())
}

fn query_member(deps: Deps, member: String) -> StdResult<MemberResponse> {
    let group = GROUP.load(deps.storage)?;
    let voter_addr = deps.api.addr_validate(&member)?;
    let weight = group.is_member(&deps.querier, &voter_addr, None)?;

    Ok(MemberResponse { weight })
}

fn list_members(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<MemberListResponse> {
    let group = GROUP.load(deps.storage)?;
    let members = group
        .list_members(&deps.querier, start_after, limit)?
        .into_iter()
        .map(|member| Member {
            addr: member.addr,
            weight: member.weight,
        })
        .collect();
    Ok(MemberListResponse { members })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    if msg.id != INIT_GROUP_REPLY_ID {
        return Err(ContractError::InvalidReplyID {});
    }

    let reply = parse_reply_instantiate_data(msg);
    match reply {
        Ok(res) => {
            let group =
                Cw4Contract(deps.api.addr_validate(&res.contract_address).map_err(|_| {
                    ContractError::InvalidGroup {
                        addr: res.contract_address.clone(),
                    }
                })?);

            GROUP.save(deps.storage, &group)?;

            Ok(Response::default().add_attribute("action", "reply_on_success"))
        }
        Err(_) => Err(ContractError::ReplyOnSuccess {}),
    }
}
