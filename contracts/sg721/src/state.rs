use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CreatorInfo {
    // address for the individual or cw4-group
    pub creator: Addr,
    pub creator_share: u64,
}

// extend contract storage with creator info
pub type Extension = CreatorInfo;

pub const CREATOR: Item<CreatorInfo> = Item::new("creator");
