use cosmwasm_std::{Addr, Empty};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw20_ics20::state::ChannelInfo;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct Config {
    pub default_timeout: u64,
}
pub const CONFIG: Item<Config> = Item::new("ics721_config");

/// static info on one channel that doesn't change
pub const CHANNEL_INFO: Map<&str, ChannelInfo> = Map::new("channel_info");

/// Indexed by (channel_id, contract_addr, token_id)
/// Keeps track of all NFTs that have passed through this channel.
// pub const CHANNEL_STATE: Map<(&str, &str, &str), Empty> = Map::new("channel_state");

/// indexed by (channel_id, class_id)
pub const CHANNEL_STATE: Map<(&str, &str), ChannelState> = Map::new("channel_state");

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ChannelState {
    pub token_ids: Vec<String>,
}
