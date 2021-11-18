use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub count: i32,
    pub owner: Addr,
}

pub const STATE: Item<State> = Item::new("state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Creators {
    // address for the cw4-group that represents the creators and their ownership weights
    pub group: Addr,
    // share of each sale for the total group
    // individual shares are handled by weights in the cw4-group
    pub share: u64,
}

pub const CREATORS: Map<&str, Creators> = Map::new("creators");
