use cosmwasm_std::{Addr, Coin, Timestamp};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub num_members: u32,
    pub unit_price: Coin,
    pub per_address_limit: u32,
    pub member_limit: u32,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const WHITELIST: Map<Addr, bool> = Map::new("wl");
