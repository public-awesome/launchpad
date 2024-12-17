use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp};
use cw_storage_plus::{Item, Map};
use sg4::Status;
use token_merge_factory::msg::MintToken;

#[cw_serde]
pub struct ConfigExtension {
    pub admin: Addr,
    pub base_token_uri: String,
    pub num_tokens: u32,
    pub start_time: Timestamp,
    pub per_address_limit: u32,
    pub mint_tokens: Vec<MintToken>,
}

#[cw_serde]
pub struct MinterConfig<T> {
    pub factory: Addr,
    pub collection_code_id: u64,
    pub extension: T,
}

pub type Config = MinterConfig<ConfigExtension>;

pub const CONFIG: Item<Config> = Item::new("config");
pub const SG721_ADDRESS: Item<Addr> = Item::new("sg721_address");
// map of index position and token id
pub const MINTABLE_TOKEN_POSITIONS: Map<u32, u32> = Map::new("mt");
pub const MINTABLE_NUM_TOKENS: Item<u32> = Item::new("mintable_num_tokens");
pub const MINTER_ADDRS: Map<&Addr, u32> = Map::new("ma");
pub const RECEIVED_TOKENS: Map<(&Addr, String), u32> = Map::new("rt");
/// Holds the status of the minter. Can be changed with on-chain governance proposals.
pub const STATUS: Item<Status> = Item::new("status");
