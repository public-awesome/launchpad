use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Timestamp};
use cw_storage_plus::Item;

#[cw_serde]
pub struct Stage {
    pub name: String,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub mint_price: Coin,
    pub per_address_limit: u32,
    pub mint_count_limit: Option<u32>,
}
#[cw_serde]
pub struct Config {
    pub stages: Vec<Stage>,
}

#[cw_serde]
pub struct AdminList {
    pub admins: Vec<Addr>,
    pub mutable: bool,
}

impl AdminList {
    pub fn is_admin(&self, addr: impl AsRef<str>) -> bool {
        let addr = addr.as_ref();
        self.admins.iter().any(|a| a.as_ref() == addr)
    }

    pub fn can_modify(&self, addr: &str) -> bool {
        self.mutable && self.is_admin(addr)
    }
}

pub const ADMIN_LIST: Item<AdminList> = Item::new("admin_list");
pub const CONFIG: Item<Config> = Item::new("config");
pub const MERKLE_ROOTS: Item<Vec<String>> = Item::new("merkle_roots");
pub const MERKLE_TREE_URIS: Item<Vec<String>> = Item::new("merkle_tree_uris");
