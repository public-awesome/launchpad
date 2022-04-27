use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::Item;
use sg_marketplace::MarketplaceContract;

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Config {
    pub marketplace: MarketplaceContract,
}

// unique items
pub const CONFIG: Item<Config> = Item::new("config");
