use cosmwasm_std::{Addr, Coin, Timestamp};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sg3::MinterConfig;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigExtension {
    pub admin: Addr,
    pub base_token_uri: String,
    pub num_tokens: u32,
    pub unit_price: Coin,
    pub initial_price: Coin,
    pub whitelist: Option<Addr>,
    pub start_time: Timestamp,
    pub per_address_limit: u32,
}
pub type Config = MinterConfig<ConfigExtension>;

pub const CONFIG: Item<Config> = Item::new("config");
pub const SG721_ADDRESS: Item<Addr> = Item::new("sg721_address");
// map of index position and token id
pub const MINTABLE_TOKEN_POSITIONS: Map<u32, u32> = Map::new("mt");
pub const MINTABLE_NUM_TOKENS: Item<u32> = Item::new("mintable_num_tokens");
pub const MINTER_ADDRS: Map<Addr, u32> = Map::new("ma");
