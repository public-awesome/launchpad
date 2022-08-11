#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, StdResult, WasmMsg};
use cw2::set_contract_version;
use cw_utils::must_pay;
use sg1::checked_fair_burn;
use sg2::query::Sg2QueryMsg;
use sg_std::NATIVE_DENOM;

use crate::error::ContractError;
use crate::msg::{
    BaseMinterCreateMsg, BaseSudoMsg, BaseUpdateParamsMsg, ExecuteMsg, InstantiateMsg,
    ParamsResponse, Response, SubMsg, SudoMsg,
};
use crate::state::SUDO_PARAMS;
use sg_controllers::{handle_reply, query_minter_status, update_params, upsert_minter_status};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:vending-factory";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Can only be called by governance
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    SUDO_PARAMS.save(deps.storage, &msg.params)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateMinter(msg) => execute_create_base_minter(deps, env, info, msg),
    }
}

pub fn execute_create_base_minter(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: BaseMinterCreateMsg,
) -> Result<Response, ContractError> {
    must_pay(&info, NATIVE_DENOM)?;

    let params = SUDO_PARAMS.load(deps.storage)?;

    let mut res = Response::new();
    checked_fair_burn(&info, params.creation_fee.amount.u128(), None, &mut res)?;

    let wasm_msg = WasmMsg::Instantiate {
        admin: Some(info.sender.to_string()),
        code_id: params.code_id,
        msg: to_binary(&msg)?,
        funds: vec![],
        label: format!("BaseMinter-{}", msg.collection_params.name),
    };
    let submsg = SubMsg::reply_on_success(wasm_msg, params.code_id);

    Ok(res
        .add_attribute("action", "create_minter")
        .add_submessage(submsg))
}

/// Reply callback triggered from creation above (minter init)
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    handle_reply(deps, msg).map_err(|e| e.into())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, env: Env, msg: BaseSudoMsg) -> Result<Response, ContractError> {
    match msg {
        SudoMsg::UpdateParams(params_msg) => sudo_update_params(deps, env, *params_msg),
        SudoMsg::UpdateMinterStatus {
            minter,
            verified,
            blocked,
        } => upsert_minter_status(deps, minter, verified, blocked)
            .map_err(|_| ContractError::MinterFactoryError {}),
    }
}

/// Only governance can update contract params
pub fn sudo_update_params(
    deps: DepsMut,
    _env: Env,
    param_msg: BaseUpdateParamsMsg,
) -> Result<Response, ContractError> {
    let mut params = SUDO_PARAMS.load(deps.storage)?;

    update_params(&mut params, param_msg)?;

    SUDO_PARAMS.save(deps.storage, &params)?;

    Ok(Response::new().add_attribute("action", "sudo_update_params"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: Sg2QueryMsg) -> StdResult<Binary> {
    match msg {
        Sg2QueryMsg::Params {} => to_binary(&query_params(deps)?),
        Sg2QueryMsg::MinterStatus { minter } => to_binary(&query_minter_status(deps, minter)?),
    }
}

fn query_params(deps: Deps) -> StdResult<ParamsResponse> {
    let params = SUDO_PARAMS.load(deps.storage)?;
    Ok(ParamsResponse { params })
}
