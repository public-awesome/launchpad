use cosmwasm_std::{Addr, Decimal};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Collection {
    pub contract_uri: String,
    pub royalties: RoyaltyInfo,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RoyaltyInfo {
    pub creator: Addr,
    /// fallback to creator address if doesn't exist
    pub creator_payment_address: Option<Addr>,
    /// fallback to owner address if doesn't exist
    pub owner_payment_address: Option<Addr>,
    pub creator_share: Decimal,
    pub owner_share: Decimal,
}

pub const COLLECTION: Item<Collection> = Item::new("collection");
