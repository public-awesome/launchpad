use cosmwasm_std::{Addr, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use crate::{
    helpers::validators::map_validate,
    msg::{AdminListResponse, CanExecuteResponse},
    state::ADMIN_LIST,
    ContractError,
};

pub fn execute_update_admins(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admins: Vec<String>,
) -> Result<Response, ContractError> {
    let mut cfg = ADMIN_LIST.load(deps.storage)?;
    if !cfg.can_modify(info.sender.as_ref()) {
        Err(ContractError::Unauthorized {})
    } else {
        cfg.admins = map_validate(deps.api, &admins)?;
        ADMIN_LIST.save(deps.storage, &cfg)?;

        let res = Response::new().add_attribute("action", "update_admins");
        Ok(res)
    }
}

pub fn can_execute(deps: &DepsMut, sender: Addr) -> Result<Addr, ContractError> {
    let cfg = ADMIN_LIST.load(deps.storage)?;
    let can = cfg.is_admin(&sender);
    if !can {
        return Err(ContractError::Unauthorized {});
    }
    Ok(sender)
}

pub fn execute_freeze(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let mut cfg = ADMIN_LIST.load(deps.storage)?;
    if !cfg.can_modify(info.sender.as_ref()) {
        Err(ContractError::Unauthorized {})
    } else {
        cfg.mutable = false;
        ADMIN_LIST.save(deps.storage, &cfg)?;

        let res = Response::new().add_attribute("action", "freeze");
        Ok(res)
    }
}

pub fn query_admin_list(deps: Deps) -> StdResult<AdminListResponse> {
    let cfg = ADMIN_LIST.load(deps.storage)?;
    Ok(AdminListResponse {
        admins: cfg.admins.into_iter().map(|a| a.into()).collect(),
        mutable: cfg.mutable,
    })
}

pub fn query_can_execute(deps: Deps, sender: &str) -> StdResult<CanExecuteResponse> {
    let cfg = ADMIN_LIST.load(deps.storage)?;
    let can = cfg.is_admin(deps.api.addr_validate(sender)?);
    Ok(CanExecuteResponse { can_execute: can })
}
