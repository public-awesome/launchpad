use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin, Decimal, Timestamp};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub base_token_uri: String,
    pub num_tokens: u32,
    pub sg721_code_id: u64,
    pub unit_price: Coin,
    pub whitelist: Option<Addr>,
    pub start_time: Timestamp,
    pub per_address_limit: u32,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const SG721_ADDRESS: Item<Addr> = Item::new("sg721_address");
// map of index position and token id
pub const MINTABLE_TOKEN_POSITIONS: Map<u32, u32> = Map::new("mt");
pub const MINTABLE_NUM_TOKENS: Item<u32> = Item::new("mintable_num_tokens");
pub const MINTER_ADDRS: Map<Addr, u32> = Map::new("ma");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Params {
    pub max_token_limit: u32,
    pub max_per_address_limit: u32,
    pub min_mint_price: u128,
    pub airdrop_mint_price: u128,
    pub mint_fee_percent: Decimal,
    pub airdrop_mint_fee_percent: Decimal,
    pub shuffle_fee: u128,
}

pub const PARAMS: Item<Params> = Item::new("params");
