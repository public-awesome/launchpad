use crate::{ContractError, Sg721Contract};

use cosmwasm_std::{DepsMut, Empty, Env, Event, Response};
use cw721::traits::StateFactory;
use cw721::{
    msg::CollectionMetadataExtensionMsg, query::Cw721Query,
    DefaultOptionCollectionMetadataExtension, DefaultOptionCollectionMetadataExtensionMsg,
    DefaultOptionNftMetadataExtension, DefaultOptionNftMetadataExtensionMsg, RoyaltyInfo,
};
use cw_storage_plus::Item;
#[allow(deprecated)]
use sg721::{CollectionInfo, RoyaltyInfoResponse};

/// Migrates collection info (sg721) into new collection metadata extension (cw721)
#[allow(deprecated)]
pub fn upgrade(deps: DepsMut, env: &Env, response: Response) -> Result<Response, ContractError> {
    let contract = Sg721Contract::<
        DefaultOptionNftMetadataExtension,
        DefaultOptionNftMetadataExtensionMsg,
        DefaultOptionCollectionMetadataExtension,
        DefaultOptionCollectionMetadataExtensionMsg,
        Empty,
    >::default();
    let event = Event::new("migrate-3.1.0");
    // migrate only in case collection metadata is not set
    let mut collection_metadata = contract
        .parent
        .query_collection_metadata(deps.as_ref(), env)?;
    let event = match collection_metadata.extension.clone() {
        Some(_) => event,
        None => {
            let legacy_collection_info_store: Item<CollectionInfo<RoyaltyInfo>> =
                Item::new("collection_info");
            let legacy_collection_info = legacy_collection_info_store.load(deps.storage)?;
            let collection_metadata_msg = CollectionMetadataExtensionMsg::<RoyaltyInfoResponse> {
                description: Some(legacy_collection_info.description),
                explicit_content: legacy_collection_info.explicit_content,
                external_link: legacy_collection_info.external_link,
                image: Some(legacy_collection_info.image),
                start_trading_time: legacy_collection_info.start_trading_time,
                royalty_info: legacy_collection_info.royalty_info.map(|r| r.into()),
            };
            let updated_collection_metadata_extension_result =
                collection_metadata_msg.create(Some(deps.as_ref()), Some(env), None, None);
            let updated_collection_metadata_extension =
                updated_collection_metadata_extension_result?;
            collection_metadata.extension = Some(updated_collection_metadata_extension);
            contract
                .parent
                .config
                .collection_metadata
                .save(deps.storage, &collection_metadata)?;
            event
        }
    };
    Ok(response.add_event(event))
}
