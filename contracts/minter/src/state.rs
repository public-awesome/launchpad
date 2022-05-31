use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin, Timestamp, Storage, StdResult};
use cw_storage_plus::{Item, Map, UniqueIndex, IndexList, Index, IndexedMap};

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

pub fn increment_tokens(storage: &mut dyn Storage) -> StdResult<u32> {
    let val = token_count(storage)? + 1;
    MINTABLE_NUM_TOKENS.save(storage, &val)?;
    Ok(val)
}

pub fn decrement_tokens(storage: &mut dyn Storage) -> StdResult<u32> {
    let val = token_count(storage)? - 1;
    MINTABLE_NUM_TOKENS.save(storage, &val)?;
    Ok(val)
}

pub const MINTER_ADDRS: Map<Addr, u32> = Map::new("ma");

type TokenKey = u32;
type TokenId = u32;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenInfo{
    pub key: TokenKey,
    pub id: TokenId
}

pub struct TokenIndices<'a> {
    pub token_ids: UniqueIndex<'a, TokenId, TokenInfo, TokenKey>,
}

impl<'a> IndexList<TokenInfo> for TokenIndices<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<TokenInfo>> + '_> {
        let v: Vec<&dyn Index<TokenInfo>> = vec![&self.token_ids];
        Box::new(v.into_iter())
    }
}

pub fn tokens<'a>() -> IndexedMap<'a, TokenKey, TokenInfo, TokenIndices<'a>> {
    let indexes = TokenIndices {
        token_ids: UniqueIndex::new(|d: &TokenInfo| d.id, "tokens__token_ids"),
    };
    IndexedMap::new("tokens", indexes)
}