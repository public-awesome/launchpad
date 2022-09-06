use cw4::Cw4Contract;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Config {
    // Total weight and members are queried from this contract
    pub group_addr: Cw4Contract,
}

// unique items
pub const CONFIG: Item<Config> = Item::new("config");
