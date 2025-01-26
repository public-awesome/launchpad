use crate::{ContractError, Sg721Contract};

use cosmwasm_std::{DepsMut, Empty, Env, Event, Response};
use cw721::execute::update_collection_info;
use cw721::msg::CollectionInfoMsg;
use cw721::{
    msg::CollectionExtensionMsg, traits::Cw721Query, DefaultOptionalCollectionExtension,
    DefaultOptionalCollectionExtensionMsg, DefaultOptionalNftExtension,
    DefaultOptionalNftExtensionMsg, RoyaltyInfo,
};
use cw_storage_plus::Item;
#[allow(deprecated)]
use sg721::{CollectionInfo, RoyaltyInfoResponse};

/// Migrates collection info (sg721) into new collection extension (cw721)
#[allow(deprecated)]
pub fn upgrade(deps: DepsMut, env: &Env, response: Response) -> Result<Response, ContractError> {
    let contract = Sg721Contract::<
        DefaultOptionalNftExtension,
        DefaultOptionalNftExtensionMsg,
        DefaultOptionalCollectionExtension,
        DefaultOptionalCollectionExtensionMsg,
        Empty,
        Empty,
        Empty,
    >::default();
    let event = Event::new("migrate-3.1.0");
    // migrate only in case collection metadata is not set
    let collection_metadata = contract
        .parent
        .query_collection_info_and_extension(deps.as_ref())?;
    let event = match collection_metadata.extension.clone() {
        Some(_) => event,
        None => {
            let legacy_collection_info_store: Item<CollectionInfo<RoyaltyInfo>> =
                Item::new("collection_info");
            let legacy_collection_info = legacy_collection_info_store.load(deps.storage)?;
            let collection_metadata_extension_msg = CollectionExtensionMsg::<RoyaltyInfoResponse> {
                description: Some(legacy_collection_info.description),
                explicit_content: legacy_collection_info.explicit_content,
                external_link: legacy_collection_info.external_link,
                image: Some(legacy_collection_info.image),
                start_trading_time: legacy_collection_info.start_trading_time,
                royalty_info: legacy_collection_info.royalty_info.map(|r| r.into()),
            };
            let collection_metadata_msg = CollectionInfoMsg {
                name: Some(collection_metadata.name),
                symbol: Some(collection_metadata.symbol),
                extension: Some(collection_metadata_extension_msg.clone()),
            };

            update_collection_info::<
                DefaultOptionalCollectionExtension,
                DefaultOptionalCollectionExtensionMsg,
                Empty,
            >(deps, None, env.into(), collection_metadata_msg)?;
            event
        }
    };
    Ok(response.add_event(event))
}
