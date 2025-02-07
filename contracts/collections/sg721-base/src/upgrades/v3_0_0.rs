use crate::ContractError;

use cosmwasm_std::{DepsMut, Empty, Env, Event, Response};
use cw721_base::Extension;

pub fn upgrade(deps: DepsMut, _env: &Env, response: Response) -> Result<Response, ContractError> {
    let cw17_res = cw721_base::upgrades::v0_17::migrate::<Extension, Empty, Empty, Empty>(deps)
        .map_err(|e| ContractError::MigrationError(e.to_string()))?;

    let mut event = Event::new("migrate-3.0.0");
    event = event.add_attributes(cw17_res.attributes);

    Ok(response.add_event(event))
}
