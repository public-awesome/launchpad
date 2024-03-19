use crate::{
    entry::{CONTRACT_NAME, CONTRACT_VERSION},
    ContractError, Sg721Contract,
};

use cosmwasm_std::{DepsMut, Empty, Env, Event, Response};
use cw721::{
    msg::Cw721MigrateMsg, DefaultOptionCollectionMetadataExtension,
    DefaultOptionCollectionMetadataExtensionMsg, DefaultOptionNftMetadataExtension,
    DefaultOptionNftMetadataExtensionMsg,
};
use cw721_base::execute::Cw721Execute;

/// Migrates cw721 states:
/// (1) legacy creator and minter migration, now using cw-ownable
/// - v0.15 and v0.16: dedicated minter store
/// - v0.17 and v0.18: minter stored using cw-ownable
/// (2) legacy contract info migration -> collection metadata
/// - before v0.19 there was only contract info
/// - now we have collection metadata with optional extension
/// (3) optional creator and minter reset (as passed in Cw721MigrateMsg::WithUpdate)
/// (4) contract name and version
///
/// Note: migration is only executed in case new stores are empty! It is safe calling these on any version.
pub fn upgrade(
    deps: DepsMut,
    env: &Env,
    response: Response,
    msg: Cw721MigrateMsg,
) -> Result<Response, ContractError> {
    let contract = Sg721Contract::<
        DefaultOptionNftMetadataExtension,
        DefaultOptionNftMetadataExtensionMsg,
        DefaultOptionCollectionMetadataExtension,
        DefaultOptionCollectionMetadataExtensionMsg,
        Empty,
    >::default();
    // cw721 migration covers these versions: 0.18. 0.17, 0.16 and 0.15
    let cw721_res = contract
        .parent
        .migrate(deps, env.clone(), msg, CONTRACT_NAME, CONTRACT_VERSION)
        .map_err(|e| ContractError::MigrationError(e.to_string()))?;

    let mut event = Event::new("migrate-3.0.0");
    event = event.add_attributes(cw721_res.attributes);

    Ok(response.add_event(event))
}
