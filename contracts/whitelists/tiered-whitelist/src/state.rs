use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Timestamp};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Stage {
    pub name: String,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub mint_price: Coin,
    pub per_address_limit: u32,
}

#[cw_serde]
pub struct Config {
    pub stages: Vec<Stage>,
    pub num_members: u32,
    pub member_limit: u32,
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

pub const WHITELIST_STAGES: Map<(u32, Addr), bool> = Map::new("wl_stages");

pub const MEMBER_COUNT: Map<u32, u32> = Map::new("member_count");
