#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure, ensure_eq, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, WasmMsg,
};
use cw2::set_contract_version;
use cw_utils::must_pay;
use semver::Version;
use sg1::{checked_fair_burn, transfer_funds_to_launchpad_dao};
use sg2::msg::UpdateMinterParamsMsg;
use sg2::query::{AllowedCollectionCodeIdResponse, AllowedCollectionCodeIdsResponse, Sg2QueryMsg};
use sg2::MinterParams;
use sg_utils::NATIVE_DENOM;

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
    let params = SUDO_PARAMS.load(deps.storage)?;
    must_pay(&info, &params.creation_fee.denom)?;
    must_be_allowed_collection(deps.as_ref(), msg.collection_params.code_id)?;

    must_not_be_frozen(&params)?;

    let mut res = Response::new();
    if params.creation_fee.denom == NATIVE_DENOM {
        checked_fair_burn(&info, params.creation_fee.amount.u128(), None, &mut res)?;
    } else {
        transfer_funds_to_launchpad_dao(
            &info,
            params.creation_fee.amount.u128(),
            &params.creation_fee.denom,
            &mut res,
        )?;
    }

    let msg = WasmMsg::Instantiate {
        admin: Some(info.sender.to_string()),
        code_id: params.code_id,
        msg: to_json_binary(&msg)?,
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

pub fn must_not_be_frozen<T>(params: &MinterParams<T>) -> Result<(), ContractError> {
    ensure!(!params.frozen, ContractError::Frozen {});
    Ok(())
}

pub fn must_pay_exact_amount<T>(
    params: &MinterParams<T>,
    info: &MessageInfo,
    accepted_denom: &str,
) -> Result<(), ContractError> {
    must_pay(info, accepted_denom)?;
    // `must_pay` checks if the denom is ok and if there is only 1 denom sent so the below is safe
    ensure!(
        info.funds[0].amount == params.creation_fee.amount,
        ContractError::InvalidCreationFeeAmount {}
    );
    Ok(())
}

pub fn must_be_allowed_collection(deps: Deps, code_id: u64) -> Result<(), ContractError> {
    let res = query_allowed_collection_code_id(deps, code_id)?;
    if !res.allowed {
        return Err(ContractError::InvalidCollectionCodeId { code_id });
    }
    Ok(())
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

    if let Some(frozen) = param_msg.frozen {
        params.frozen = frozen;
    }

    if let Some(creation_fee) = param_msg.creation_fee {
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

    // add new code ids, then rm code ids
    if let Some(add_sg721_code_ids) = param_msg.add_sg721_code_ids {
        for code_id in add_sg721_code_ids {
            params.allowed_sg721_code_ids.push(code_id);
        }
    }
    params.allowed_sg721_code_ids.dedup();
    if let Some(rm_sg721_code_ids) = param_msg.rm_sg721_code_ids {
        for code_id in rm_sg721_code_ids {
            params.allowed_sg721_code_ids.retain(|&x| x != code_id);
        }
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
        Sg2QueryMsg::Params {} => to_json_binary(&query_params(deps)?),
        Sg2QueryMsg::AllowedCollectionCodeIds {} => {
            to_json_binary(&query_allowed_collection_code_ids(deps)?)
        }
        Sg2QueryMsg::AllowedCollectionCodeId(code_id) => {
            to_json_binary(&query_allowed_collection_code_id(deps, code_id)?)
        }
    }
}

fn query_params(deps: Deps) -> StdResult<ParamsResponse> {
    let params = SUDO_PARAMS.load(deps.storage)?;
    Ok(ParamsResponse { params })
}

fn query_allowed_collection_code_ids(deps: Deps) -> StdResult<AllowedCollectionCodeIdsResponse> {
    let params = SUDO_PARAMS.load(deps.storage)?;
    let code_ids = params.allowed_sg721_code_ids;
    Ok(AllowedCollectionCodeIdsResponse { code_ids })
}

fn query_allowed_collection_code_id(
    deps: Deps,
    code_id: u64,
) -> StdResult<AllowedCollectionCodeIdResponse> {
    let params = SUDO_PARAMS.load(deps.storage)?;
    let code_ids = params.allowed_sg721_code_ids;
    let allowed = code_ids.contains(&code_id);
    Ok(AllowedCollectionCodeIdResponse { allowed })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(
    deps: DepsMut,
    _env: Env,
    msg: Option<BaseUpdateParamsMsg>,
) -> Result<Response, ContractError> {
    let prev_contract_info = cw2::get_contract_version(deps.storage)?;
    let prev_contract_name: String = prev_contract_info.contract;
    let prev_contract_version: Version = prev_contract_info
        .version
        .parse()
        .map_err(|_| StdError::generic_err("Unable to retrieve previous contract version"))?;

    let new_version: Version = CONTRACT_VERSION
        .parse()
        .map_err(|_| StdError::generic_err("Invalid contract version"))?;

    if prev_contract_name != CONTRACT_NAME {
        return Err(StdError::generic_err("Cannot migrate to a different contract").into());
    }

    if prev_contract_version > new_version {
        return Err(StdError::generic_err("Cannot migrate to a previous contract version").into());
    }

    if let Some(msg) = msg {
        let mut params = SUDO_PARAMS.load(deps.storage)?;

        update_params(&mut params, msg)?;

        SUDO_PARAMS.save(deps.storage, &params)?;
    }

    Ok(Response::new().add_attribute("action", "migrate"))
}
