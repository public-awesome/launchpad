use cw20_ics20::ibc::Ics20Ack;
use cw20_ics20::state::ChannelInfo;
use cw721::Cw721ExecuteMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, Binary, ContractResult, DepsMut, Empty, Env,
    IbcBasicResponse, IbcChannel, IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg,
    IbcEndpoint, IbcOrder, IbcPacket, IbcPacketAckMsg, IbcPacketReceiveMsg, IbcPacketTimeoutMsg,
    IbcReceiveResponse, Reply, Response, SubMsg, WasmMsg,
};

use crate::error::{ContractError, Never};
use crate::state::{CHANNEL_INFO, CHANNEL_STATE};

pub const ICS721_VERSION: &str = "ics721-1";
pub const ICS721_ORDERING: IbcOrder = IbcOrder::Unordered;

// TODO: need to define proto for chain to parse this?
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct Ics721Packet {
    /// uniquely identifies the collection to which the NFT belongs
    /// the cw721 collection contract address
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
        ContractResult::Ok(_) => Response::new(),
        ContractResult::Err(err) => {
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
    let split_denom: Vec<&str> = voucher_class_id.splitn(3, '/').collect();
    if split_denom.len() != 3 {
        return Err(ContractError::NoForeignTokens {});
    }
    // a few more sanity checks
    if split_denom[0] != remote_endpoint.port_id {
        return Err(ContractError::FromOtherPort {
            port: split_denom[0].into(),
        });
    }
    if split_denom[1] != remote_endpoint.channel_id {
        return Err(ContractError::FromOtherChannel {
            channel: split_denom[1].into(),
        });
    }

    Ok(split_denom[2])
}

// this does the work of ibc_packet_receive, we wrap it to turn errors into acknowledgements
fn do_ibc_packet_receive(deps: DepsMut, packet: &IbcPacket) -> Result<Ics721Packet, ContractError> {
    let msg: Ics721Packet = from_binary(&packet.data)?;
    let channel = packet.dest.channel_id.clone();

    // If the token originated on the remote chain, it looks like "stars1.....".
    // TODO: handle tokens originating from a remote chain
    // https://github.com/public-awesome/contracts/issues/56

    // If it originated on our chain, it looks like "port/channel/stars1.....".
    let contract_addr = parse_voucher_contract_address(&msg.class_id, &packet.src)?;

    // We received an NFT with a class_id that looks like "port/channel/stars1..."
    // This means that it originated on this chain, so we have to check the channel
    // state and make sure we have a record of sending it.
    // If we find it, remove it from state and return Ok.
    // If we don't find it, return Err.

    let state =
        CHANNEL_STATE.may_load(deps.storage, (&channel, contract_addr, &msg.token_ids[0]))?;
    match state {
        Some(_) => CHANNEL_STATE.remove(deps.storage, (&channel, contract_addr, &msg.token_ids[0])),
        None => {
            return Err(ContractError::NoSuchNft {
                class_id: msg.class_id,
            })
        }
    };

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
    CHANNEL_STATE.save(
        deps.storage,
        (&channel, &msg.class_id, &msg.token_ids[0]),
        &Empty {},
    )?;

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

// TODO: The standard allows sending more than one token at a time.
// We are just sending one for now.
fn send_tokens(
    contract_addr: &str,
    token_ids: Vec<String>,
    _token_uris: Vec<String>,
    recipient: String,
) -> SubMsg {
    // TODO: need a `TransferFullNft` or `TransferRemoteNft` that includes token_uri
    let msg = Cw721ExecuteMsg::TransferNft {
        recipient,
        token_id: token_ids[0].clone(),
    };
    let exec = WasmMsg::Execute {
        contract_addr: contract_addr.to_string(),
        msg: to_binary(&msg).unwrap(),
        funds: vec![],
    };
    SubMsg::reply_on_error(exec, SEND_NFT_ID)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_helpers::*;

    use crate::contract::query_channel;
    use cosmwasm_std::testing::mock_env;
    use cosmwasm_std::{to_vec, IbcAcknowledgement, IbcEndpoint, IbcTimeout, Timestamp};

    #[test]
    fn check_ack_json() {
        let success = Ics20Ack::Result(b"1".into());
        let fail = Ics20Ack::Error("bad coin".into());

        let success_json = String::from_utf8(to_vec(&success).unwrap()).unwrap();
        assert_eq!(r#"{"result":"MQ=="}"#, success_json.as_str());

        let fail_json = String::from_utf8(to_vec(&fail).unwrap()).unwrap();
        assert_eq!(r#"{"error":"bad coin"}"#, fail_json.as_str());
    }

    #[test]
    fn check_packet_json() {
        let packet = Ics721Packet::new(
            "stars1zedxv25ah8fksmg2lzrndrpkvsjqgk4zt5ff7n",
            Some("https://metadata-url.com/my-metadata"),
            vec!["1", "2", "3"],
            vec![
                "https://metadata-url.com/my-metadata1",
                "https://metadata-url.com/my-metadata2",
                "https://metadata-url.com/my-metadata3",
            ],
            "stars1zedxv25ah8fksmg2lzrndrpkvsjqgk4zt5ff7n",
            "wasm1fucynrfkrt684pm8jrt8la5h2csvs5cnldcgqc",
        );
        // Example message generated from the SDK
        let expected = r#"{"class_id":"stars1zedxv25ah8fksmg2lzrndrpkvsjqgk4zt5ff7n","class_uri":"https://metadata-url.com/my-metadata","token_ids":["1","2","3"],"token_uris":["https://metadata-url.com/my-metadata1","https://metadata-url.com/my-metadata2","https://metadata-url.com/my-metadata3"],"sender":"stars1zedxv25ah8fksmg2lzrndrpkvsjqgk4zt5ff7n","receiver":"wasm1fucynrfkrt684pm8jrt8la5h2csvs5cnldcgqc"}"#;

        let encdoded = String::from_utf8(to_vec(&packet).unwrap()).unwrap();
        assert_eq!(expected, encdoded.as_str());
    }

    fn _cw721_transfer(token_id: String, address: &str, recipient: &str) -> SubMsg {
        let msg = Cw721ExecuteMsg::TransferNft {
            token_id,
            recipient: recipient.into(),
        };
        let exec = WasmMsg::Execute {
            contract_addr: address.into(),
            msg: to_binary(&msg).unwrap(),
            funds: vec![],
        };
        SubMsg::reply_on_error(exec, SEND_NFT_ID)
    }

    fn mock_sent_packet(
        my_channel: &str,
        class_id: &str,
        token_ids: Vec<&str>,
        token_uris: Vec<&str>,
        sender: &str,
    ) -> IbcPacket {
        let data = Ics721Packet {
            class_id: class_id.into(),
            class_uri: None,
            token_ids: token_ids
                .iter()
                .map(|&s| s.to_string())
                .collect::<Vec<String>>(),
            token_uris: token_uris
                .iter()
                .map(|&s| s.to_string())
                .collect::<Vec<String>>(),
            sender: sender.to_string(),
            receiver: "remote-rcpt".to_string(),
        };
        IbcPacket::new(
            to_binary(&data).unwrap(),
            IbcEndpoint {
                port_id: CONTRACT_PORT.to_string(),
                channel_id: my_channel.to_string(),
            },
            IbcEndpoint {
                port_id: REMOTE_PORT.to_string(),
                channel_id: "channel-1234".to_string(),
            },
            2,
            IbcTimeout::with_timestamp(Timestamp::from_seconds(1665321069)),
        )
    }

    fn mock_receive_packet(
        my_channel: &str,
        class_id: &str,
        token_ids: Vec<&str>,
        token_uris: Vec<&str>,
        receiver: &str,
    ) -> IbcPacket {
        let data = Ics721Packet {
            // this is returning a foreign (our) token, thus class_id is <port>/<channel>/<contract_addr>
            class_id: format!("{}/{}/{}", REMOTE_PORT, "channel-1234", class_id),
            class_uri: None,
            token_ids: token_ids
                .iter()
                .map(|&s| s.to_string())
                .collect::<Vec<String>>(),
            token_uris: token_uris
                .iter()
                .map(|&s| s.to_string())
                .collect::<Vec<String>>(),
            sender: "remote-sender".to_string(),
            receiver: receiver.to_string(),
        };
        println!("Packet class_id: {}", &data.class_id);
        IbcPacket::new(
            to_binary(&data).unwrap(),
            IbcEndpoint {
                port_id: REMOTE_PORT.to_string(),
                channel_id: "channel-1234".to_string(),
            },
            IbcEndpoint {
                port_id: CONTRACT_PORT.to_string(),
                channel_id: my_channel.to_string(),
            },
            3,
            Timestamp::from_seconds(1665321069).into(),
        )
    }

    #[test]
    fn send_receive_cw721() {
        let send_channel = "channel-9";
        let mut deps = setup(&["channel-1", "channel-7", send_channel]);

        let contract_addr = "collection-addr";
        let token_ids = vec!["1", "2", "3"];
        let token_uris = vec![
            "https://metadata-url.com/my-metadata1",
            "https://metadata-url.com/my-metadata2",
            "https://metadata-url.com/my-metadata3",
        ];

        // prepare some mock packets
        let sent_packet = mock_sent_packet(
            send_channel,
            contract_addr,
            token_ids.clone(),
            token_uris.clone(),
            "local-sender",
        );
        let recv_packet = mock_receive_packet(
            send_channel,
            contract_addr,
            token_ids,
            token_uris,
            "local-rcpt",
        );

        let msg = IbcPacketReceiveMsg::new(recv_packet);
        // cannot receive this class_id yet
        // TODO: but should be able to after implementing sending to other cw721 contracts
        let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();
        println!("{:?}", res.attributes);
        assert!(res.messages.is_empty());
        let ack: Ics20Ack = from_binary(&res.acknowledgement).unwrap();
        let no_such_nft = Ics20Ack::Error(
            ContractError::NoSuchNft {
                class_id: "class_id".to_string(),
            }
            .to_string(),
        );
        assert_eq!(ack, no_such_nft);

        // we get a success cache (ack) for a send
        let msg = IbcPacketAckMsg::new(IbcAcknowledgement::new(ack_success()), sent_packet);
        let res = ibc_packet_ack(deps.as_mut(), mock_env(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        // query channel state|_|
        let _state = query_channel(deps.as_ref(), send_channel.to_string()).unwrap();
        // assert_eq!(state.balances, vec![Amount::cw20(987654321, cw721_addr)]);
        // assert_eq!(state.total_sent, vec![Amount::cw20(987654321, cw721_addr)]);

        // // cannot receive more than we sent
        // let msg = IbcPacketReceiveMsg::new(recv_high_packet);
        // let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();
        // assert!(res.messages.is_empty());
        // let ack: Ics20Ack = from_binary(&res.acknowledgement).unwrap();
        // assert_eq!(ack, no_funds);

        // // we can receive less than we sent
        // let msg = IbcPacketReceiveMsg::new(recv_packet);
        // let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();
        // assert_eq!(1, res.messages.len());
        // assert_eq!(
        //     cw20_payment(876543210, cw721_addr, "local-rcpt"),
        //     res.messages[0]
        // );
        // let ack: Ics20Ack = from_binary(&res.acknowledgement).unwrap();
        // matches!(ack, Ics20Ack::Result(_));

        // // query channel state
        // let state = query_channel(deps.as_ref(), send_channel.to_string()).unwrap();
        // assert_eq!(state.balances, vec![Amount::cw20(111111111, cw721_addr)]);
        // assert_eq!(state.total_sent, vec![Amount::cw20(987654321, cw721_addr)]);
    }
}
