use cw20_ics20::state::ChannelInfo;
use cw721::Cw721ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::ChannelState;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    // Default timeout for ics721 packets, specified in seconds
    pub default_timeout: u64,
}

// This is the message we accept via Receive
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TransferMsg {
    // The local channel to send the packets on
    pub channel: String,
    // uniquely identifies the collection to which the NFT belongs
    pub class_id: String,
    // https://docs.opensea.io/docs/contract-level-metadata
    pub class_uri: Option<String>,
    // uniquely identifies NFTs within the collection that is being transferred
    pub token_ids: Vec<String>,
    // https://docs.opensea.io/docs/metadata-standards
    pub token_uris: Vec<String>,
    // The remote address to send to
    pub remote_address: String,
    // How long the packet lives in seconds. If not specified, use default_timeout.
    pub timeout: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    // Accepts an NFT from a cw721 contract, then transfers it over IBC
    Receive(Cw721ReceiveMsg),
    // Transfer an NFT over IBC to another chain
    Transfer(TransferMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Return the port ID bound by this contract. Returns PortResponse
    Port {},
    // Show all channels we have connected to. Return type is ListChannelsResponse.
    ListChannels {},
    // Returns the details of the name channel, error if not created.
    // Return type: ChannelResponse.
    Channel { id: String },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ChannelResponse {
    /// Information on the channel's connection
    pub info: ChannelInfo,
    pub tokens: Vec<ChannelState>,
    // pub tokens_received: Vec<>,
    // pub tokens_sent: Vec<>
}
