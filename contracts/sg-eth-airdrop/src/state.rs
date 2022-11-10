use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
pub struct Config {
    pub admin: Addr,
    pub claim_msg_plaintext: String,
    pub airdrop_amount: u128,
    pub whitelist_address: Option<String>,
    pub minter_address: Addr,
}

pub const CONFIG: Item<Config> = Item::new("cfg");
pub const ELIGIBLE_ETH_ADDRS: Map<&str, bool> = Map::new("eth_address");
