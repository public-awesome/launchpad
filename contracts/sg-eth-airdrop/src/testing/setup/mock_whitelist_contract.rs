use crate::error::ContractError;
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, StdResult};
use sg_std::Response;
use sg_whitelist::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    mut _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let res = Response::new();
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AddMembers(_) => execute_add_members(),
        _ => Err(ContractError::InvalidReplyID {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    to_binary("mock")
}

fn execute_add_members() -> Result<Response, ContractError> {
    let res = Response::new();
    Ok(res)
}
