use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub per_address_limit: u32,
    pub minter_contract: Option<Addr>,
    pub mint_discount_bps: Option<u64>,
}

impl Config {
    pub fn mint_discount(&self) -> Option<Decimal> {
        self.mint_discount_bps
            .map(|v| Decimal::percent(v) / Uint128::from(100u128))
    }
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const TOTAL_ADDRESS_COUNT: Item<u64> = Item::new("total_address_count");
// Holds all addresses and mint count
pub const WHITELIST: Map<&str, u32> = Map::new("wl");
