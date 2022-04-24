use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// use cw4::Cw4Contract;
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Config {
    pub marketplace: MarketplaceContract,
}

// unique items
pub const CONFIG: Item<Config> = Item::new("config");

// TODO:: remove after open sourcing marketplace
/// MarketplaceContract is a wrapper around Addr that provides a lot of helpers
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MarketplaceContract(pub Addr);

impl MarketplaceContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }
}
