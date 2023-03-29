use cosmwasm_std::{Addr, Empty};
use cw_storage_plus::Item;
use serde::{de::DeserializeOwned, Serialize};
use sg721::{CollectionInfo, RoyaltyInfo};
use sg_std::StargazeMsgWrapper;
use std::ops::Deref;

type Parent<'a, T> = cw721_base::Cw721Contract<'a, T, StargazeMsgWrapper, Empty, Empty>;
pub struct Sg721Contract<'a, T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    pub parent: Parent<'a, T>,
    pub minter: Item<'a, Addr>,

    pub collection_info: Item<'a, CollectionInfo<RoyaltyInfo>>,

    /// Instantiate set to false by the minter, then true by creator to freeze collection info
    pub frozen_collection_info: Item<'a, bool>,
}

impl<'a, T> Default for Sg721Contract<'a, T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    fn default() -> Self {
        Sg721Contract {
            parent: cw721_base::Cw721Contract::default(),
            collection_info: Item::new("collection_info"),
            frozen_collection_info: Item::new("frozen_collection_info"),
            minter: Item::new("minter_address"),
        }
    }
}

impl<'a, T> Deref for Sg721Contract<'a, T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    type Target = Parent<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}
