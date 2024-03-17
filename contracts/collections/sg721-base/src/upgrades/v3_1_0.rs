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
    let royalty_updated_at = env.block.time.minus_seconds(60 * 60 * 24); // 24 hours ago

    contract
        .royalty_updated_at
        .save(deps.storage, &royalty_updated_at)?;

    let migrate_msg = Cw721MigrateMsg::WithUpdate {
        minter: None,
        creator: None,
    };
    // cw721 migration allows all versions: 0.18. 0.17, 0.16 and older
    let cw721_res = contract
        .parent
        .migrate(
            deps,
            env.clone(),
            migrate_msg,
            CONTRACT_NAME,
            CONTRACT_VERSION,
        )
        .map_err(|e| ContractError::MigrationError(e.to_string()))?;

    let mut event = Event::new("migrate-3.1.0")
        .add_attribute("royalty-updated", royalty_updated_at.to_string());
    event = event.add_attributes(cw721_res.attributes);

    Ok(response.add_event(event))
}
