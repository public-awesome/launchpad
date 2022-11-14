use crate::error::ContractError;
use crate::msg::SudoMsg;
use crate::state::SUDO_PARAMS;
use cosmwasm_std::{entry_point, Addr, DepsMut, Env, MessageInfo, Storage};
use sg_std::Response;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, _env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    let api = deps.api;

    match msg {
        SudoMsg::AddOperator { operator } => sudo_add_operator(deps, api.addr_validate(&operator)?),
        SudoMsg::RemoveOperator { operator } => {
            sudo_remove_operator(deps, api.addr_validate(&operator)?)
        }
    }
}

pub fn sudo_add_operator(deps: DepsMut, operator: Addr) -> Result<Response, ContractError> {
    let mut params = SUDO_PARAMS.load(deps.storage)?;
    if !params.operators.iter().any(|o| o == &operator) {
        params.operators.push(operator.clone());
    } else {
        return Err(ContractError::OperatorAlreadyRegistered {});
    }
    SUDO_PARAMS.save(deps.storage, &params)?;
    let res = Response::new()
        .add_attribute("action", "add_operator")
        .add_attribute("operator", operator);
    Ok(res)
}

pub fn sudo_remove_operator(deps: DepsMut, operator: Addr) -> Result<Response, ContractError> {
    let mut params = SUDO_PARAMS.load(deps.storage)?;
    if let Some(i) = params.operators.iter().position(|o| o == &operator) {
        params.operators.remove(i);
    } else {
        return Err(ContractError::OperatorNotRegistered {});
    }
    SUDO_PARAMS.save(deps.storage, &params)?;
    let res = Response::new()
        .add_attribute("action", "remove_operator")
        .add_attribute("operator", operator);
    Ok(res)
}

pub fn only_operator(store: &dyn Storage, info: &MessageInfo) -> Result<Addr, ContractError> {
    let params = SUDO_PARAMS.load(store)?;
    if !params
        .operators
        .iter()
        .any(|a| a.as_ref() == info.sender.as_ref())
    {
        return Err(ContractError::UnauthorizedOperator {});
    }
    Ok(info.sender.clone())
}
