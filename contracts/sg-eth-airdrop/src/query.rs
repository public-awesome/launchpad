use crate::{state::CONFIG, ContractError};
use cosmwasm_std::{Deps, DepsMut, StdResult};
use vending_minter::helpers::MinterContract;
use whitelist_generic::helpers::WhitelistGenericContract;

pub fn query_collection_whitelist(deps: &DepsMut) -> Result<String, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let minter_addr = config.minter_address;
    let config = MinterContract(minter_addr).config(&deps.querier);
    match config {
        Ok(result) => match result.whitelist {
            Some(wl) => Ok(wl),
            None => Err(ContractError::CollectionWhitelistMinterNotSet {}),
        },
        Err(_) => Err(ContractError::CollectionWhitelistMinterNotSet {}),
    }
}

pub fn query_airdrop_is_eligible(deps: Deps, eth_address: String) -> StdResult<bool> {
    let config = CONFIG.load(deps.storage)?;
    match config.whitelist_address {
        Some(address) => WhitelistGenericContract(deps.api.addr_validate(&address)?)
            .includes(&deps.querier, eth_address),
        None => Err(cosmwasm_std::StdError::NotFound {
            kind: "Whitelist Contract".to_string(),
        }),
    }
}
