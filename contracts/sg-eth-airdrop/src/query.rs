use crate::{state::CONFIG, ContractError};
use cosmwasm_std::DepsMut;
use vending_minter::helpers::MinterContract;

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
