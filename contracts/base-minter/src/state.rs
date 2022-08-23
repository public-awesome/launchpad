use cosmwasm_std::{Addr, Empty, StdResult, Storage};
use cw_storage_plus::Item;
use sg3::MinterConfig;

pub type Config = MinterConfig<Empty>;

/// Initial configuration of the minter
pub const CONFIG: Item<Config> = Item::new("config");

/// This is saved after handling a reply in instantiation. Therefore it's not in `Config`.
pub const COLLECTION_ADDRESS: Item<Addr> = Item::new("collection_address");

/// This keeps track of the token index for the token_ids
pub const TOKEN_INDEX: Item<u64> = Item::new("token_index");

pub fn increment_token_index(store: &mut dyn Storage) -> StdResult<u64> {
    let val = TOKEN_INDEX.may_load(store)?.unwrap_or_default() + 1;
    TOKEN_INDEX.save(store, &val)?;
    Ok(val)
}
