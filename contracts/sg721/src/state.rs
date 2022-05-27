use cosmwasm_std::{Addr, Decimal};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sg_controllers::Hooks;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollectionInfo<T> {
    pub creator: String,
    pub description: String,
    pub image: String,
    pub external_link: Option<String>,
    pub royalty_info: Option<T>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RoyaltyInfo {
    pub payment_address: Addr,
    pub share: Decimal,
}

pub const COLLECTION_INFO: Item<CollectionInfo<RoyaltyInfo>> = Item::new("collection_info");

pub const TRANSFER_HOOKS: Hooks = Hooks::new("transfer-hooks");
