#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Storage,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{InstantiateMsg, ParamResponseu32, QueryMsg, SudoMsg};
use crate::state::U32;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:paramstore";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// TODO: make sure this can only be sudo initialized
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: SudoMsg,
) -> Result<Response, ContractError> {
    match msg {
        SudoMsg::UpdateParamu32 {
            contract_name,
            key,
            value,
        } => {
            let param = U32.update(
                deps.storage,
                (contract_name.clone(), key.clone()),
                |old| -> StdResult<_> { Ok(old.unwrap_or_default() + value) },
            )?;
            Ok(Response::new()
                .add_attribute("action", "update_param")
                .add_attribute("contract_name", contract_name)
                .add_attribute("key", key)
                .add_attribute("value", param.to_string()))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetParamu32 { contract_name, key } => {
            to_binary(&query_param_u32(deps.storage, contract_name, key)?)
        }
    }
}

fn query_param_u32(
    store: &dyn Storage,
    contract_name: String,
    key: String,
) -> StdResult<ParamResponseu32> {
    let value = U32.load(store, (contract_name, key))?;
    Ok(ParamResponseu32 { value })
}
