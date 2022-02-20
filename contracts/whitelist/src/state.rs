use cw_storage_plus::Item;
use cw_utils::Expiration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub start_time: Expiration,
    pub end_time: Expiration,
}

pub const CONFIG: Item<Config> = Item::new("config");
