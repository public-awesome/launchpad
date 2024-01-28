use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, StdResult, Storage, Timestamp};
use cw_storage_plus::{Item, Map};

use open_edition_factory::types::NftData;
use sg4::{MinterConfig, Status};

#[cw_serde]
pub struct ConfigExtension {
    pub admin: Addr,
    pub payment_address: Option<Addr>,
    pub nft_data: NftData,
    pub start_time: Timestamp,
    pub end_time: Option<Timestamp>,
    pub per_address_limit: u32,
    pub num_tokens: Option<u32>,
}
pub type Config = MinterConfig<ConfigExtension>;

pub const CONFIG: Item<Config> = Item::new("config");
pub const SG721_ADDRESS: Item<Addr> = Item::new("sg721_address");
pub const MINTER_ADDRS: Map<&Addr, u32> = Map::new("ma");

/// This keeps track of the mint count
pub const TOTAL_MINT_COUNT: Item<u32> = Item::new("total_mint_count");

pub const MINTABLE_NUM_TOKENS: Item<u32> = Item::new("mintable_num_tokens");

/// Holds the status of the minter. Can be changed with on-chain governance proposals.
pub const STATUS: Item<Status> = Item::new("status");

/// This keeps track of the token index for the token_ids
pub const TOKEN_INDEX: Item<u64> = Item::new("token_index");

pub fn increment_token_index(store: &mut dyn Storage) -> StdResult<u64> {
    let val = TOKEN_INDEX.may_load(store)?.unwrap_or_default() + 1;
    TOKEN_INDEX.save(store, &val)?;
    Ok(val)
}
