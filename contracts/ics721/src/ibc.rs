use cw20_ics20::ibc::Ics20Ack;
use cw20_ics20::state::ChannelInfo;
use cw721::Cw721ExecuteMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, Binary, DepsMut, Empty, Env, IbcBasicResponse,
    IbcChannel, IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcEndpoint, IbcOrder,
    IbcPacket, IbcPacketAckMsg, IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcReceiveResponse,
    Reply, Response, SubMsg, SubMsgResult, WasmMsg,
};

use crate::error::{ContractError, Never};
use crate::state::{CHANNEL_INFO, CHANNEL_STATE};

pub const ICS721_VERSION: &str = "ics721-1";
pub const ICS721_ORDERING: IbcOrder = IbcOrder::Unordered;

#[cfg(test)]
#[path = "ibc_test.rs"]
mod ibc_test;

// TODO: need to define proto for chain to parse this?
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct Ics721Packet {
    /// uniquely identifies the collection to which the NFT belongs
    /// the sg721 collection contract address
    pub class_id: String,
    // a link for class/contract-level metadata
    /// https://docs.opensea.io/docs/contract-level-metadata
    pub class_uri: Option<String>,
    /// uniquely identifies NFTs within the collection that is being transferred
    pub token_ids: Vec<String>,
    /// https://docs.opensea.io/docs/metadata-standards
    pub token_uris: Vec<String>,
    pub sender: String,
    pub receiver: String,
}

impl Ics721Packet {
    pub fn new(
        class_id: &str,
        class_uri: Option<&str>,
        token_ids: Vec<&str>,
        token_uris: Vec<&str>,
        sender: &str,
        receiver: &str,
    ) -> Self {
        Ics721Packet {
            class_id: class_id.to_string(),
            class_uri: class_uri.map(str::to_string),
            token_ids: token_ids
                .iter()
                .map(|&s| s.to_string())
                .collect::<Vec<String>>(),
            token_uris: token_uris
                .iter()
                .map(|&s| s.to_string())
                .collect::<Vec<String>>(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
        }
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        // TODO:
        // https://github.com/public-awesome/contracts/issues/53
        // validate class_id is a contract address
        // validate class_uri is a uri if it exists
        // validate same number of token_ids and token_uris
        Ok(())
    }
}

// TODO: copy pasta, make parent public
// create a serialized success message
fn ack_success() -> Binary {
    let res = Ics20Ack::Result(b"1".into());
    to_binary(&res).unwrap()
}

// TODO: copy pasta, make parent public
// create a serialized error message
fn ack_fail(err: String) -> Binary {
    let res = Ics20Ack::Error(err);
    to_binary(&res).unwrap()
}

const SEND_NFT_ID: u64 = 1338;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    if reply.id != SEND_NFT_ID {
        return Err(ContractError::UnknownReplyId { id: reply.id });
    }
    let res = match reply.result {
        SubMsgResult::Ok(_) => Response::new(),
        SubMsgResult::Err(err) => {
            // encode an acknowledgement error
            Response::new().set_data(ack_fail(err))
        }
    };
    Ok(res)
}

// IBC entrypoint 1
#[cfg_attr(not(feature = "library"), entry_point)]
/// enforces ordering and versioning constraints
pub fn ibc_channel_open(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelOpenMsg,
) -> Result<(), ContractError> {
    enforce_order_and_version(msg.channel(), msg.counterparty_version())?;
    Ok(())
}

// IBC entrypoint 2
#[cfg_attr(not(feature = "library"), entry_point)]
/// record the channel in CHANNEL_INFO
pub fn ibc_channel_connect(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> Result<IbcBasicResponse, ContractError> {
    // we need to check the counter party version in try and ack (sometimes here)
    enforce_order_and_version(msg.channel(), msg.counterparty_version())?;

    let channel: IbcChannel = msg.into();
    let info = ChannelInfo {
        id: channel.endpoint.channel_id,
        counterparty_endpoint: channel.counterparty_endpoint,
        connection_id: channel.connection_id,
    };
    CHANNEL_INFO.save(deps.storage, &info.id, &info)?;

    Ok(IbcBasicResponse::default())
}

fn enforce_order_and_version(
    channel: &IbcChannel,
    counterparty_version: Option<&str>,
) -> Result<(), ContractError> {
    if channel.version != ICS721_VERSION {
        return Err(ContractError::InvalidIbcVersion {
            version: channel.version.clone(),
        });
    }
    if let Some(version) = counterparty_version {
        if version != ICS721_VERSION {
            return Err(ContractError::InvalidIbcVersion {
                version: version.to_string(),
            });
        }
    }
    if channel.order != ICS721_ORDERING {
        return Err(ContractError::OnlyOrderedChannel {});
    }
    Ok(())
}

// IBC entrypoint 3
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_close(
    _deps: DepsMut,
    _env: Env,
    _channel: IbcChannelCloseMsg,
) -> Result<IbcBasicResponse, ContractError> {
    // TODO: what to do here?
    // we will have locked funds that need to be returned somehow
    unimplemented!();
}

// IBC entrypoint 4
#[cfg_attr(not(feature = "library"), entry_point)]
/// We should not return an error if possible, but rather an acknowledgement of failure
pub fn ibc_packet_receive(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, Never> {
    let packet = msg.packet;

    let res = match do_ibc_packet_receive(deps, &packet) {
        Ok(msg) => {
            // this cannot fail as we did it before..
            // TODO: find a way to not call this again?
            let contract_addr = parse_voucher_contract_address(&msg.class_id, &packet.src).unwrap();

            let attributes = vec![
                attr("action", "receive"),
                attr("sender", &msg.sender),
                attr("receiver", &msg.receiver),
                attr("contract_address", contract_addr.to_string()),
                attr("token_ids", msg.token_ids.join(",")),
                attr("success", "true"),
            ];
            let msg = send_tokens(contract_addr, msg.token_ids, msg.token_uris, msg.receiver);
            IbcReceiveResponse::new()
                .set_ack(ack_success())
                .add_submessage(msg)
                .add_attributes(attributes)
        }
        Err(err) => IbcReceiveResponse::new()
            .set_ack(ack_fail(err.to_string()))
            .add_attributes(vec![
                attr("action", "receive"),
                attr("success", "false"),
                attr("error", err.to_string()),
            ]),
    };

    // send the tokens to the requested recipient
    Ok(res)
}

/// Returns local contract_address if it is an encoded voucher from the expected endpoint
/// Otherwise, error
fn parse_voucher_contract_address<'a>(
    voucher_class_id: &'a str,
    remote_endpoint: &IbcEndpoint,
) -> Result<&'a str, ContractError> {
    let split_class_id: Vec<&str> = voucher_class_id.splitn(3, '/').collect();
    if split_class_id.len() != 3 {
        // only accept NFTs originating from this chain
        // https://github.com/public-awesome/contracts/issues/56
        return Err(ContractError::NoForeignTokens {});
    }
    // a few more sanity checks
    if split_class_id[0] != remote_endpoint.port_id {
        return Err(ContractError::FromOtherPort {
            port: split_class_id[0].into(),
        });
    }
    if split_class_id[1] != remote_endpoint.channel_id {
        return Err(ContractError::FromOtherChannel {
            channel: split_class_id[1].into(),
        });
    }

    Ok(split_class_id[2])
}

// this does the work of ibc_packet_receive, we wrap it to turn errors into acknowledgements
fn do_ibc_packet_receive(deps: DepsMut, packet: &IbcPacket) -> Result<Ics721Packet, ContractError> {
    let msg: Ics721Packet = from_binary(&packet.data)?;
    let channel = packet.dest.channel_id.clone();

    // If the token originated on another chain, it looks like "juno1.....".
    // TODO: handle tokens originating from a remote chain
    // https://github.com/public-awesome/contracts/issues/56

    // If it originated on our chain, it looks like "port/channel/stars1.....".
    let contract_addr = parse_voucher_contract_address(&msg.class_id, &packet.src)?;

    // We received an NFT with a class_id that looks like "port/channel/stars1..."
    // This means that it originated on this chain, so we have to check the channel
    // state and make sure we have a record of sending it.
    // If we find it, remove it from state and return Ok.
    // If we don't find it, return Err.
    for token_id in &msg.token_ids {
        let state = CHANNEL_STATE.may_load(deps.storage, (&channel, contract_addr, token_id))?;
        match state {
            Some(_) => (),
            None => {
                return Err(ContractError::NoSuchNft {
                    class_id: msg.class_id,
                })
            }
        };
    }
    for token_id in &msg.token_ids {
        CHANNEL_STATE.remove(deps.storage, (&channel, contract_addr, token_id));
    }
    Ok(msg)
}

// IBC entrypoint 5
#[cfg_attr(not(feature = "library"), entry_point)]
/// check if success or failure and update balance, or return funds
pub fn ibc_packet_ack(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketAckMsg,
) -> Result<IbcBasicResponse, ContractError> {
    // TODO: trap error like in receive?
    let ics20msg: Ics20Ack = from_binary(&msg.acknowledgement.data)?;
    match ics20msg {
        Ics20Ack::Result(_) => on_packet_success(deps, msg.original_packet),
        Ics20Ack::Error(err) => on_packet_failure(deps, msg.original_packet, err),
    }
}

// IBC entrypoint 6
#[cfg_attr(not(feature = "library"), entry_point)]
/// return fund to original sender (same as failure in ibc_packet_ack)
pub fn ibc_packet_timeout(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketTimeoutMsg,
) -> Result<IbcBasicResponse, ContractError> {
    // TODO: trap error like in receive?
    let packet = msg.packet;
    on_packet_failure(deps, packet, "timeout".to_string())
}

// update the info stored on this (channel, contract_addr) index
fn on_packet_success(deps: DepsMut, packet: IbcPacket) -> Result<IbcBasicResponse, ContractError> {
    let msg: Ics721Packet = from_binary(&packet.data)?;
    let attributes = vec![
        attr("action", "acknowledge"),
        attr("sender", &msg.sender),
        attr("receiver", &msg.receiver),
        attr("contract_addr", &msg.class_id),
        attr("success", "true"),
    ];

    let channel = packet.src.channel_id;
    for token in &msg.token_ids {
        CHANNEL_STATE.save(deps.storage, (&channel, &msg.class_id, token), &Empty {})?;
    }
    Ok(IbcBasicResponse::new().add_attributes(attributes))
}

// return the tokens to sender
fn on_packet_failure(
    _deps: DepsMut,
    packet: IbcPacket,
    err: String,
) -> Result<IbcBasicResponse, ContractError> {
    let msg: Ics721Packet = from_binary(&packet.data)?;
    let contract_addr = parse_voucher_contract_address(&msg.class_id, &packet.src)?;
    let attributes = vec![
        attr("action", "acknowledge"),
        attr("sender", &msg.sender),
        attr("receiver", &msg.receiver),
        attr("contract_addr", contract_addr),
        attr("success", "false"),
        attr("error", err),
    ];

    let msg = send_tokens(contract_addr, msg.token_ids, msg.token_uris, msg.sender);
    Ok(IbcBasicResponse::new()
        .add_attributes(attributes)
        .add_submessage(msg))
}

fn send_tokens(
    contract_addr: &str,
    token_ids: Vec<String>,
    _token_uris: Vec<String>,
    recipient: String,
) -> SubMsg {
    // TODO: need a `TransferFullNft` or `TransferRemoteNft` that includes token_uri
    let mut msgs: Vec<Cw721ExecuteMsg> = Vec::new();
    for token_id in token_ids {
        let msg = Cw721ExecuteMsg::TransferNft {
            recipient: recipient.clone(),
            token_id: token_id.clone(),
        };
        msgs.push(msg);
    }
    let exec = WasmMsg::Execute {
        contract_addr: contract_addr.to_string(),
        msg: to_binary(&msgs).unwrap(),
        funds: vec![],
    };
    SubMsg::reply_on_error(exec, SEND_NFT_ID)
}
