use cw_storage_plus::Item;
use serde::{de::DeserializeOwned, Serialize};
use sg721::{CollectionInfo, RoyaltyInfo};
use sg_std::StargazeMsgWrapper;
use std::ops::Deref;

pub const FROZEN_TOKEN_METADATA: Item<bool> = Item::new("frozen_token_metadata");

type Parent<'a, T> = cw721_base::Cw721Contract<'a, T, StargazeMsgWrapper>;
pub struct Sg721UpdatableContract<'a, T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    pub parent: Parent<'a, T>,

    pub collection_info: Item<'a, CollectionInfo<RoyaltyInfo>>,

    /// Instantiate set to false by the minter, then true by creator to freeze collection info
    pub frozen_collection_info: Item<'a, bool>,
}

impl<'a, T> Default for Sg721UpdatableContract<'a, T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    fn default() -> Self {
        Sg721UpdatableContract {
            parent: cw721_base::Cw721Contract::default(),
            collection_info: Item::new("collection_info"),
            frozen_collection_info: Item::new("frozen_collection_info"),
        }
    }
}

impl<'a, T> Deref for Sg721UpdatableContract<'a, T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    type Target = Parent<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}
