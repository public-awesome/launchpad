use cosmwasm_std::{Addr, Decimal};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Extensions to cw721-base that will be saved in TokenInfo
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Extension {
    pub creator: Addr,
    pub royalties: Option<RoyaltyInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RoyaltyInfo {
    pub creator_share: Decimal,
    pub owner_share: Decimal,
}

pub const EXTENSION: Item<Extension> = Item::new("extension");
