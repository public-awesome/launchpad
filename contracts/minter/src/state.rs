use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin, StdResult, Storage, Timestamp};
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
pub const MINTABLE_NUM_TOKENS: Item<u32> = Item::new("mintable_num_tokens");

pub fn token_count(storage: &dyn Storage) -> StdResult<u32> {
    Ok(MINTABLE_NUM_TOKENS.may_load(storage)?.unwrap_or_default())
}

// decrements mintable tokens as supply is minted or burned
pub fn decrement_tokens(storage: &mut dyn Storage) -> StdResult<u32> {
    let val = token_count(storage)? - 1;
    MINTABLE_NUM_TOKENS.save(storage, &val)?;
    Ok(val)
}

pub const MINTER_ADDRS: Map<Addr, u32> = Map::new("ma");
pub const MINTABLE_TOKEN_IDS: Item<Vec<u32>> = Item::new("mintable_token_ids");

type TokenPosition = usize;
type TokenId = u32;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenIndexMapping {
    pub position: TokenPosition,
    pub id: TokenId,
}
