use cosmwasm_std::{Addr, Empty};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub count: i32,
    pub owner: Addr,
}

pub const STATE: Item<State> = Item::new("state");

// sg721 <> creator mapping
pub const CREATORS: Map<&str, Creator> = Map::new("creators");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Creator {
    // address for the individual or..
    // cw4-group that represents the creators and their ownership weights
    pub creator: Addr,
    // share of each sale for the total group
    // individual shares are handled by weights in the cw4-group
    pub share: u64,
}
// use creator as the extention in token info
pub type Extension = Creator;
