use cosmwasm_std::Addr;
use cw_storage_plus::Map;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CreatorInfo {
    // address for the individual or cw4-group
    pub creator: Addr,
    pub creator_share: u64,
}

// sg721 <> creator mapping
pub const CREATORS: Map<&Addr, CreatorInfo> = Map::new("creators");

// extend contract storage with creator info
pub type Extension = CreatorInfo;
