use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Config {
    pub admin : Addr,
    pub claim_msg_plaintext: String

}// unique items

pub const CONFIG: Item<Config> = Item::new("cfg");
pub const ELIGIBLE_ETH_ADDRS: Map<&Addr, bool> = Map::new("eth_address");