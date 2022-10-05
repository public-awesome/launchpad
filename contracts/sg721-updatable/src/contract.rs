#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, QueryMsg};
use sg721::InstantiateMsg;

use cw721_base::Extension;
use cw_utils::nonpayable;
use sg721_base::Sg721Contract;
pub type Sg721UpdatableContract<'a> = Sg721Contract<'a, Extension>;
use sg_std::Response;

const CONTRACT_NAME: &str = "crates.io:sg721-updatable";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // instantiate is the same as sg721 base contracts
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let res = Sg721UpdatableContract::default().instantiate(deps, env, info, msg)?;
    Ok(res
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    unimplemented!()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}

#[cfg(test)]
mod tests {}
