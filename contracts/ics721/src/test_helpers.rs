#![cfg(test)]

// TODO: much of this is copy-pasta from cw20-ics20 since the lib is not made public
// and ICS20 constants are hardcoded. Make generic and move into testing package.
use crate::contract::instantiate;
use crate::ibc::{ibc_channel_connect, ibc_channel_open};
use crate::ibc::{ICS721_ORDERING, ICS721_VERSION};
use cw20_ics20::state::ChannelInfo;

use cosmwasm_std::testing::{
    mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_std::{
    DepsMut, IbcChannel, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcEndpoint, OwnedDeps,
};

use crate::msg::InstantiateMsg;

pub const DEFAULT_TIMEOUT: u64 = 3600; // 1 hour,
pub const CONTRACT_PORT: &str = "ibc:wasm1234567890abcdef";
pub const REMOTE_PORT: &str = "transfer-nft";

pub struct ChannelSetupData<'a> {
    pub source_channel: &'a str,
    pub dest_channel: &'a str,
    pub connection: &'a str,
}

pub fn mock_channel(channel_data: ChannelSetupData) -> IbcChannel {
    IbcChannel::new(
        IbcEndpoint {
            channel_id: channel_data.source_channel.into(),
            port_id: CONTRACT_PORT.into(),
        },
        IbcEndpoint {
            channel_id: channel_data.dest_channel.into(),
            port_id: REMOTE_PORT.into(),
        },
        ICS721_ORDERING,
        ICS721_VERSION,
        channel_data.connection,
    )
}

pub fn mock_channel_info(channel_setup: ChannelSetupData) -> ChannelInfo {
    ChannelInfo {
        id: channel_setup.source_channel.to_string(),
        counterparty_endpoint: IbcEndpoint {
            port_id: REMOTE_PORT.into(),
            channel_id: channel_setup.dest_channel.to_string(),
        },
        connection_id: channel_setup.connection.into(),
    }
}

// we simulate instantiate and ack here
pub fn add_channel(mut deps: DepsMut, channel_setup: ChannelSetupData) {
    let channel = mock_channel(channel_setup);
    let open_msg = IbcChannelOpenMsg::new_init(channel.clone());
    ibc_channel_open(deps.branch(), mock_env(), open_msg).unwrap();
    let connect_msg = IbcChannelConnectMsg::new_ack(channel, ICS721_VERSION);
    ibc_channel_connect(deps.branch(), mock_env(), connect_msg).unwrap();
}

pub fn setup(
    channel_setup_data: &[ChannelSetupData],
) -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
    let mut deps = mock_dependencies();

    // instantiate an empty contract
    let instantiate_msg = InstantiateMsg {
        default_timeout: DEFAULT_TIMEOUT,
    };
    let info = mock_info(&String::from("anyone"), &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    for channel_setup in channel_setup_data {
        add_channel(
            deps.as_mut(),
            ChannelSetupData {
                source_channel: channel_setup.source_channel,
                dest_channel: channel_setup.dest_channel,
                connection: channel_setup.connection,
            },
        )
    }
    deps
}
