use crate::{ContractError, Sg721Contract};

use cosmwasm_std::{DepsMut, Env, Event};
use cw721_base::Extension;
use sg_std::Response;

pub fn upgrade(deps: DepsMut, env: &Env, response: Response) -> Result<Response, ContractError> {
    let contract = Sg721Contract::<Extension>::default();

    let royalty_updated = env.block.time.minus_seconds(60 * 60 * 24); // 24 hours ago

    contract
        .royalty_updated
        .save(deps.storage, &royalty_updated)?;

    let event =
        Event::new("migrate-3.1.0").add_attribute("royalty-updated", royalty_updated.to_string());

    Ok(response.add_event(event))
}
