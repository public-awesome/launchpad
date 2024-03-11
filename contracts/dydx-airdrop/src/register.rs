use crate::{state::CONFIG, ContractError};
use cosmwasm_std::DepsMut;
use cosmwasm_std::{Env, MessageInfo};
use sg_std::Response;
use crate::build_messages::{dust_and_whitelist_add};
use crate::validation::{validate_registration};

pub fn register(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    eth_address: String,
    eth_sig: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    validate_registration(
        &deps,
        info.clone(),
        eth_address.clone(),
        eth_sig,
        config.clone(),
    )?;
    let res = dust_and_whitelist_add(&deps, info, eth_address.clone())?;

    Ok(res.add_attribute("address_registered", eth_address))
}
