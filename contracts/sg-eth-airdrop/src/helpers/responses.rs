use crate::error::ContractError;
use crate::state::CONFIG;
use cosmwasm_std::{DepsMut, MessageInfo};
use sg_std::Response;

use crate::build_msg::{build_add_eth_eligible_msg, build_remove_eth_eligible_msg};

pub fn get_add_eligible_eth_response(
    deps: DepsMut,
    info: MessageInfo,
    addresses: Vec<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {
            sender: info.sender,
        });
    }
    let whitelist_address = match config.whitelist_address {
        Some(address) => address,
        None => return Err(ContractError::WhitelistContractNotSet {}),
    };
    let mut res = Response::new();
    let add_eth_msg = build_add_eth_eligible_msg(deps, addresses, whitelist_address)?;
    res = res.add_message(add_eth_msg);
    Ok(res)
}

pub fn get_remove_eligible_eth_response(
    deps: &DepsMut,
    eth_address: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let whitelist_address = match config.whitelist_address {
        Some(address) => address,
        None => return Err(ContractError::WhitelistContractNotSet {}),
    };

    let mut res = Response::new();
    let remove_eth_msg = build_remove_eth_eligible_msg(deps, eth_address, whitelist_address)?;
    res = res.add_message(remove_eth_msg);
    Ok(res)
}
