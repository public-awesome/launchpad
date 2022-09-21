#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure_eq, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, StdResult, WasmMsg,
};
use cw2::set_contract_version;
use cw_utils::must_pay;
use sg1::checked_fair_burn;
use sg2::msg::UpdateMinterParamsMsg;
use sg2::query::Sg2QueryMsg;
use sg2::MinterParams;
use sg_std::{Response, NATIVE_DENOM};

use crate::error::ContractError;
use crate::msg::{
    BaseMinterCreateMsg, BaseSudoMsg, BaseUpdateParamsMsg, ExecuteMsg, InstantiateMsg,
    ParamsResponse, SudoMsg,
};
use crate::state::SUDO_PARAMS;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg-base-factory";
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
        ExecuteMsg::CreateMinter(msg) => execute_create_minter(deps, env, info, msg),
    }
}

pub fn execute_create_minter(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: BaseMinterCreateMsg,
) -> Result<Response, ContractError> {
    must_pay(&info, NATIVE_DENOM)?;

    let params = SUDO_PARAMS.load(deps.storage)?;

    let mut res = Response::new();
    checked_fair_burn(&info, params.creation_fee.amount.u128(), None, &mut res)?;

    let msg = WasmMsg::Instantiate {
        admin: Some(info.sender.to_string()),
        code_id: params.code_id,
        msg: to_binary(&msg)?,
        funds: vec![],
        label: format!(
            "Minter-{}-{}",
            params.code_id,
            msg.collection_params.name.trim()
        ),
    };

    Ok(res
        .add_attribute("action", "create_minter")
        .add_message(msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, env: Env, msg: BaseSudoMsg) -> Result<Response, ContractError> {
    match msg {
        SudoMsg::UpdateParams(params_msg) => sudo_update_params(deps, env, *params_msg),
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

/// Base update params that can be used by other minter factories
pub fn update_params<T, C>(
    params: &mut MinterParams<C>,
    param_msg: UpdateMinterParamsMsg<T>,
) -> Result<(), ContractError> {
    params.code_id = param_msg.code_id.unwrap_or(params.code_id);

    if let Some(creation_fee) = param_msg.creation_fee {
        ensure_eq!(
            &creation_fee.denom,
            &NATIVE_DENOM,
            ContractError::InvalidDenom {}
        );
        params.creation_fee = creation_fee;
    }

    if let Some(min_mint_price) = param_msg.min_mint_price {
        ensure_eq!(
            &min_mint_price.denom,
            &NATIVE_DENOM,
            ContractError::InvalidDenom {}
        );
        params.min_mint_price = min_mint_price;
    }

    params.mint_fee_bps = param_msg.mint_fee_bps.unwrap_or(params.mint_fee_bps);

    params.max_trading_offset_secs = param_msg
        .max_trading_offset_secs
        .unwrap_or(params.max_trading_offset_secs);

    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: Sg2QueryMsg) -> StdResult<Binary> {
    match msg {
        Sg2QueryMsg::Params {} => to_binary(&query_params(deps)?),
    }
}

fn query_params(deps: Deps) -> StdResult<ParamsResponse> {
    let params = SUDO_PARAMS.load(deps.storage)?;
    Ok(ParamsResponse { params })
}
