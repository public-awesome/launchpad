use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Timestamp};
use cw_storage_plus::{Item, Map};
use sg4::{MinterConfig, Status};

#[cw_serde]
pub struct ConfigExtension {
    pub admin: Addr,
    pub payment_address: Option<Addr>,
    pub base_token_uri: String,
    pub num_tokens: u32,
    pub whitelist: Option<Addr>,
    pub start_time: Timestamp,
    pub per_address_limit: u32,
    pub discount_price: Option<Coin>,
}
pub type Config = MinterConfig<ConfigExtension>;

pub const CONFIG: Item<Config> = Item::new("config");
pub const SG721_ADDRESS: Item<Addr> = Item::new("sg721_address");
// map of index position and token id
pub const MINTABLE_TOKEN_POSITIONS: Map<u32, u32> = Map::new("mt");
pub const MINTABLE_NUM_TOKENS: Item<u32> = Item::new("mintable_num_tokens");
pub const MINTER_ADDRS: Map<&Addr, u32> = Map::new("ma");

/// Holds the status of the minter. Can be changed with on-chain governance proposals.
pub const STATUS: Item<Status> = Item::new("status");
