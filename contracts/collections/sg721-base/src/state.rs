use cosmwasm_std::Timestamp;
use cw721::traits::{Cw721CustomMsg, Cw721State};
use cw_storage_plus::Item;

use std::ops::Deref;

type Parent<
    'a,
    // Metadata defined in NftInfo (used for mint).
    TNftMetadataExtension,
    // Message passed for updating metadata.
    TNftMetadataExtensionMsg,
    // Extension defined in CollectionMetadata.
    TCollectionMetadataExtension,
    TCollectionMetadataExtensionMsg,
    // Defines for `CosmosMsg::Custom<T>` in response. Barely used, so `Empty` can be used.
    TCustomResponseMsg,
> = cw721_base::Cw721Contract<
    'a,
    TNftMetadataExtension,
    TNftMetadataExtensionMsg,
    TCollectionMetadataExtension,
    TCollectionMetadataExtensionMsg,
    TCustomResponseMsg,
>;

pub struct Sg721Contract<
    'a,
    // Metadata defined in NftInfo (used for mint).
    TNftMetadataExtension,
    // Message passed for updating metadata.
    TNftMetadataExtensionMsg,
    // Extension defined in CollectionMetadata.
    TCollectionMetadataExtension,
    TCollectionMetadataExtensionMsg,
    // Defines for `CosmosMsg::Custom<T>` in response. Barely used, so `Empty` can be used.
    TCustomResponseMsg,
> where
    TNftMetadataExtension: Cw721State,
    TNftMetadataExtensionMsg: Cw721CustomMsg,
    TCollectionMetadataExtension: Cw721State,
    TCollectionMetadataExtensionMsg: Cw721CustomMsg,
{
    pub parent: Parent<
        'a,
        TNftMetadataExtension,
        TNftMetadataExtensionMsg,
        TCollectionMetadataExtension,
        TCollectionMetadataExtensionMsg,
        TCustomResponseMsg,
    >,
    /// Instantiate set to false by the minter, then true by creator to freeze collection info
    pub frozen_collection_info: Item<'a, bool>,
    pub royalty_updated_at: Item<'a, Timestamp>,
}

impl<
        'a,
        TNftMetadataExtension,
        TNftMetadataExtensionMsg,
        TCollectionMetadataExtension,
        TCollectionMetadataExtensionMsg,
        TCustomResponseMsg,
    > Default
    for Sg721Contract<
        'a,
        TNftMetadataExtension,
        TNftMetadataExtensionMsg,
        TCollectionMetadataExtension,
        TCollectionMetadataExtensionMsg,
        TCustomResponseMsg,
    >
where
    TNftMetadataExtension: Cw721State,
    TNftMetadataExtensionMsg: Cw721CustomMsg,
    TCollectionMetadataExtension: Cw721State,
    TCollectionMetadataExtensionMsg: Cw721CustomMsg,
{
    fn default() -> Self {
        Sg721Contract {
            parent: cw721_base::Cw721Contract::default(),
            frozen_collection_info: Item::new("frozen_collection_info"),
            royalty_updated_at: Item::new("royalty_updated_at"),
        }
    }
}

impl<
        'a,
        TNftMetadataExtension,
        TNftMetadataExtensionMsg,
        TCollectionMetadataExtension,
        TCollectionMetadataExtensionMsg,
        TCustomResponseMsg,
    > Deref
    for Sg721Contract<
        'a,
        TNftMetadataExtension,
        TNftMetadataExtensionMsg,
        TCollectionMetadataExtension,
        TCollectionMetadataExtensionMsg,
        TCustomResponseMsg,
    >
where
    TNftMetadataExtension: Cw721State,
    TNftMetadataExtensionMsg: Cw721CustomMsg,
    TCollectionMetadataExtension: Cw721State,
    TCollectionMetadataExtensionMsg: Cw721CustomMsg,
{
    type Target = Parent<
        'a,
        TNftMetadataExtension,
        TNftMetadataExtensionMsg,
        TCollectionMetadataExtension,
        TCollectionMetadataExtensionMsg,
        TCustomResponseMsg,
    >;

    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}
