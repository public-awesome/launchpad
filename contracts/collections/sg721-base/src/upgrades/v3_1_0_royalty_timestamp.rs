use crate::{
    entry::{CONTRACT_NAME, CONTRACT_VERSION},
    ContractError, Sg721Contract,
};
use cw721_base::execute::Cw721Execute;

use cosmwasm_std::{DepsMut, Empty, Env, Event, Response};
use cw721::{
    msg::Cw721MigrateMsg, DefaultOptionCollectionMetadataExtension,
    DefaultOptionCollectionMetadataExtensionMsg, DefaultOptionNftMetadataExtension,
    DefaultOptionNftMetadataExtensionMsg,
};

pub fn upgrade(deps: DepsMut, env: &Env, response: Response) -> Result<Response, ContractError> {
    let contract = Sg721Contract::<
        DefaultOptionNftMetadataExtension,
        DefaultOptionNftMetadataExtensionMsg,
        DefaultOptionCollectionMetadataExtension,
        DefaultOptionCollectionMetadataExtensionMsg,
        Empty,
    >::default();
    // check whether royalty timestamp already exists
    let royalty_updated_at = contract.royalty_updated_at.may_load(deps.storage)?;
    if let Some(_) = royalty_updated_at {
        // already migrated
        return Ok(response);
    }
    let royalty_updated_at = env.block.time.minus_seconds(60 * 60 * 24); // 24 hours ago

    contract
        .royalty_updated_at
        .save(deps.storage, &royalty_updated_at)?;

    let event = Event::new("migrate-3.1.0")
        .add_attribute("royalty-updated", royalty_updated_at.to_string());
    Ok(response.add_event(event))
}
