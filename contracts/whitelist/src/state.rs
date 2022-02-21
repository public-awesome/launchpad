use cosmwasm_std::{Addr, Empty};
use cw_storage_plus::{Item, Map};
use cw_utils::Expiration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub start_time: Expiration,
    pub end_time: Expiration,
    pub num_addresses: u32,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const WHITELIST: Map<Addr, Empty> = Map::new("whitelist");
