use cw_storage_plus::Item;
use sg721::{CollectionInfo, RoyaltyInfo};

pub const COLLECTION_INFO: Item<CollectionInfo<RoyaltyInfo>> = Item::new("collection_info");
