#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coins, to_binary, Addr, BankMsg, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Reply,
    Response, StdResult, SubMsg,
};
use cw2::set_contract_version;
use cw4::{Cw4Contract, Member, MemberListResponse, MemberResponse};
use cw_utils::parse_reply_instantiate_data;
use sg_std::NATIVE_DENOM;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, Group, InstantiateMsg, QueryMsg};
use crate::state::GROUP;

// version info for migration info
pub const CONTRACT_NAME: &str = "crates.io:sg-splits";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const INIT_GROUP_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let self_addr = env.contract.address;

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
    match msg {
        ExecuteMsg::Distribute {} => execute_distribute(deps, env, info),
    }
}

pub fn execute_distribute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let group = GROUP.load(deps.storage)?;

    // only a member can distribute funds
    let weight = group
        .is_member(&deps.querier, &info.sender, None)?
        .ok_or(ContractError::Unauthorized {})?;
    if weight == 0 {
        return Err(ContractError::InvalidWeight { weight });
    }

    let total_weight = checked_total_weight(&group, deps.as_ref())?;

    let funds = deps
        .querier
        .query_balance(env.contract.address, NATIVE_DENOM)?;
    if funds.amount.is_zero() {
        return Err(ContractError::NoFunds {});
    }

    let msgs = group
        .list_members(&deps.querier, None, None)?
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

fn checked_total_weight(group: &Cw4Contract, deps: Deps) -> Result<u64, ContractError> {
    let weight = group.total_weight(&deps.querier)?;
    if weight == 0 {
        return Err(ContractError::InvalidWeight { weight });
    }

    Ok(weight)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Group {} => to_binary(&query_group(deps)?),
        QueryMsg::ListMembers { start_after, limit } => {
            to_binary(&list_members(deps, start_after, limit)?)
        }
        QueryMsg::Member { address } => to_binary(&query_member(deps, address)?),
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
