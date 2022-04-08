use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub count: i32,
    pub owner: Addr,
}

pub const STATE: Item<State> = Item::new("state");

pub const PARAM_U32: Map<&str, u32> = Map::new("u32");
pub const PARAM_U64: Map<&str, u64> = Map::new("u64");
pub const PARAM_UINT128: Map<&str, Uint128> = Map::new("uint128");
pub const PARAM_ADDR: Map<&str, Addr> = Map::new("addr");
