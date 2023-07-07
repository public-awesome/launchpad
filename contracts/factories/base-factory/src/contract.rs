#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure, ensure_eq, to_binary, Binary, Deps, DepsMut, Empty, Env, Event, MessageInfo, StdError,
    StdResult, WasmMsg,
};
use cw2::set_contract_version;
use cw_utils::must_pay;
use sg1::checked_fair_burn;
use sg2::msg::UpdateMinterParamsMsg;
use sg2::query::{AllowedCollectionCodeIdResponse, AllowedCollectionCodeIdsResponse, Sg2QueryMsg};
use sg2::{MinterParams, DEFAULT_MAX_ROYALTY_BPS, DEFAULT_MAX_ROYALTY_INCREASE_RATE_BPS};
use sg_std::{Response, NATIVE_DENOM};

use crate::error::ContractError;
use crate::msg::{
    BaseMinterCreateMsg, BaseSudoMsg, BaseUpdateParamsMsg, ExecuteMsg, InstantiateMsg,
    ParamsResponse, SudoMsg,
};
use crate::state::{EARLIEST_VERSION, SUDO_PARAMS, TO_VERSION};

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

    // Check royalty rates exist
    // They must be optional to work with migrations of older contracts
    if msg.params.max_royalty_bps.is_none() {
        return Err(ContractError::RoyaltyBpsNotSet {});
    }
    if msg.params.max_royalty_increase_rate_bps.is_none() {
        return Err(ContractError::RoyaltyIncreaseRateBpsNotSet {});
    }

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
    must_be_allowed_collection(deps.as_ref(), msg.collection_params.code_id)?;

    let params = SUDO_PARAMS.load(deps.storage)?;
    must_not_be_frozen(&params)?;

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

pub fn must_not_be_frozen<T>(params: &MinterParams<T>) -> Result<(), ContractError> {
    ensure!(!params.frozen, ContractError::Frozen {});
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

    params.max_royalty_bps = if let Some(max_royalty_bps) = param_msg.max_royalty_bps {
        Some(max_royalty_bps)
    } else {
        params.max_royalty_bps
    };

    params.max_royalty_increase_rate_bps =
        if let Some(max_royalty_increase_rate_bps) = param_msg.max_royalty_increase_rate_bps {
            Some(max_royalty_increase_rate_bps)
        } else {
            params.max_royalty_increase_rate_bps
        };

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
        Sg2QueryMsg::AllowedCollectionCodeIds {} => {
            to_binary(&query_allowed_collection_code_ids(deps)?)
        }
        Sg2QueryMsg::AllowedCollectionCodeId(code_id) => {
            to_binary(&query_allowed_collection_code_id(deps, code_id)?)
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
pub fn migrate(deps: DepsMut, env: Env, msg: Empty) -> StdResult<Response> {
    try_migrate(deps, env, msg)
}

pub fn try_migrate(deps: DepsMut, _env: Env, _msg: Empty) -> StdResult<Response> {
    // make sure the correct contract is being upgraded, and it's being
    // upgraded from the correct version.

    if CONTRACT_VERSION < EARLIEST_VERSION {
        return Err(StdError::generic_err("Cannot upgrade to a previous contract version").into());
    }
    if CONTRACT_VERSION > TO_VERSION {
        return Err(StdError::generic_err("Cannot upgrade to a previous contract version").into());
    }
    // if same version return
    if CONTRACT_VERSION == TO_VERSION {
        return Ok(Response::new());
    }

    // update contract version
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, TO_VERSION)?;

    let mut params = SUDO_PARAMS.load(deps.storage)?;
    update_migrate_params(&mut params)?;
    SUDO_PARAMS.save(deps.storage, &params)?;

    let event = Event::new("migrate")
        .add_attribute("from_name", CONTRACT_NAME)
        .add_attribute("to_version", TO_VERSION)
        .add_attribute("from_version", CONTRACT_VERSION);

    Ok(Response::new().add_event(event))
}

pub fn update_migrate_params<T>(params: &mut MinterParams<T>) -> StdResult<()> {
    // set values for new fields in v3.0.0 sg2 MinterParams, can change afterwards w governance
    // max_royalty_bps: u64,
    // pub max_royalty_increase_rate_bps: u64,

    params.max_royalty_bps = Some(DEFAULT_MAX_ROYALTY_BPS);
    params.max_royalty_increase_rate_bps = Some(DEFAULT_MAX_ROYALTY_INCREASE_RATE_BPS);

    Ok(())
}
