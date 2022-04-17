#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, Deps, DepsMut, Env, IbcMsg, IbcQuery, MessageInfo, Order,
    PortIdResponse, Response, StdResult,
};
use cw2::set_contract_version;
use cw20_ics20::msg::{ListChannelsResponse, PortResponse};
use cw721::Cw721ReceiveMsg;
use cw_utils::nonpayable;

use crate::error::ContractError;
use crate::ibc::Ics721Packet;
use crate::msg::{ChannelResponse, ExecuteMsg, InstantiateMsg, QueryMsg, TransferMsg};
use crate::state::{Config, CHANNEL_INFO, CHANNEL_STATE, CONFIG};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg721-ics721";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
#[path = "contract_test.rs"]
mod contract_test;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let cfg = Config {
        default_timeout: msg.default_timeout,
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
        ExecuteMsg::Receive(msg) => execute_receive(deps, env, info, msg),
        ExecuteMsg::Transfer(msg) => execute_transfer(deps, env, msg, info.sender),
    }
}

pub fn execute_receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    wrapper: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let msg: TransferMsg = from_binary(&wrapper.msg)?;
    let api = deps.api;
    execute_transfer(deps, env, msg, api.addr_validate(&wrapper.sender)?)
}

pub fn execute_transfer(
    deps: DepsMut,
    env: Env,
    msg: TransferMsg,
    sender: Addr,
) -> Result<Response, ContractError> {
    // ensure the requested channel is registered
    if !CHANNEL_INFO.has(deps.storage, &msg.channel) {
        return Err(ContractError::NoSuchChannel { id: msg.channel });
    };

    // delta from user is in seconds
    let timeout_delta = match msg.timeout {
        Some(t) => t,
        None => CONFIG.load(deps.storage)?.default_timeout,
    };
    // timeout is in nanoseconds
    let timeout = env.block.time.plus_nanos(timeout_delta);

    // build ics721 packet
    let packet = Ics721Packet::new(
        env.contract.address.as_ref(),
        None,
        msg.token_ids
            .iter()
            .map(|s| s.as_ref())
            .collect::<Vec<&str>>(),
        msg.token_uris
            .iter()
            .map(|s| s.as_ref())
            .collect::<Vec<&str>>(),
        sender.as_ref(),
        &msg.remote_address,
    );
    packet.validate()?;

    let msg = IbcMsg::SendPacket {
        channel_id: msg.channel,
        data: to_binary(&packet)?,
        timeout: timeout.into(),
    };

    // Note: we update local state when we get ack - do not count this transfer towards anything until acked
    // similar event messages like ibctransfer module

    // send response
    let res = Response::new()
        .add_message(msg)
        .add_attribute("action", "transfer")
        .add_attribute("sender", &packet.sender)
        .add_attribute("receiver", &packet.receiver)
        .add_attribute("class_id", &packet.class_id)
        .add_attribute("token_ids", &packet.token_ids.join(","));
    Ok(res)
}

// TODO: Alot of this query code is copy pasta.
// Find a way to make it generic or put into a package.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Port {} => to_binary(&query_port(deps)?),
        QueryMsg::ListChannels {} => to_binary(&query_list(deps)?),
        QueryMsg::Channel { id } => to_binary(&query_channel(deps, id)?),
        QueryMsg::Tokens {
            channel_id,
            class_id,
        } => to_binary(&query_tokens(deps, channel_id, class_id)?),
    }
}

fn query_port(deps: Deps) -> StdResult<PortResponse> {
    let query = IbcQuery::PortId {}.into();
    let PortIdResponse { port_id } = deps.querier.query(&query)?;
    Ok(PortResponse { port_id })
}

fn query_list(deps: Deps) -> StdResult<ListChannelsResponse> {
    let channels: StdResult<Vec<_>> = CHANNEL_INFO
        .range(deps.storage, None, None, Order::Ascending)
        .map(|r| r.map(|(_, v)| v))
        .collect();
    Ok(ListChannelsResponse {
        channels: channels?,
    })
}

pub fn query_channel(deps: Deps, id: String) -> StdResult<ChannelResponse> {
    let info = CHANNEL_INFO.load(deps.storage, &id)?;
    let _class_ids: StdResult<Vec<_>> = CHANNEL_STATE
        .sub_prefix(&id)
        .range(deps.storage, None, None, Order::Ascending)
        .map(|r| {
            let (class_id_token_id, _) = r?;
            Ok(class_id_token_id.0)
        })
        .collect();

    let class_ids_resp = _class_ids;
    match class_ids_resp {
        Ok(mut class_id_vec) => Ok(ChannelResponse {
            info,
            class_ids: {
                class_id_vec.sort();
                class_id_vec.dedup();
                class_id_vec
            },
        }),
        Err(msg) => Err(msg),
    }
}

// TODO: https://github.com/public-awesome/contracts/issues/59
pub fn query_tokens(
    _deps: Deps,
    _channel_id: String,
    _class_id: String,
) -> StdResult<ChannelResponse> {
    todo!()
}
