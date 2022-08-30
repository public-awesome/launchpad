use cw_storage_plus::Item;
use serde::{de::DeserializeOwned, Serialize};
use sg721::{CollectionInfo, RoyaltyInfo};
use sg_std::StargazeMsgWrapper;

pub struct Sg721Contract<'a, T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    pub parent: cw721_base::Cw721Contract<'a, T, StargazeMsgWrapper>,

    pub collection_info: Item<'a, CollectionInfo<RoyaltyInfo>>,

    /// Set to true by the minter to indicate the minter creation process is complete
    pub ready: Item<'a, bool>,
}

impl<'a, T> Default for Sg721Contract<'a, T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    fn default() -> Self {
        Sg721Contract {
            parent: cw721_base::Cw721Contract::default(),
            collection_info: Item::new("collection_info"),
            ready: Item::new("ready"),
        }
    }
}
