use cosmwasm_std::{
    Binary, Deps, DepsMut, Env, MessageInfo, StdResult, to_binary, WasmMsg,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cw2::set_contract_version;
use cw_utils::must_pay;
use sg_std::{NATIVE_DENOM, Response};

use base_factory::contract::{must_be_allowed_collection, must_not_be_frozen, update_params};
use base_factory::ContractError as BaseContractError;
use sg1::checked_fair_burn;
use sg2::query::{AllowedCollectionCodeIdResponse, AllowedCollectionCodeIdsResponse, Sg2QueryMsg};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, OpenEditionMinterCreateMsg, OpenEditionMinterInitMsgExtension, OpenEditionUpdateParamsMsg, ParamsResponse, SudoMsg};
use crate::state::SUDO_PARAMS;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:open-edition-factory";
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
    env: Env,
    info: MessageInfo,
    mut msg: OpenEditionMinterCreateMsg,
) -> Result<Response, ContractError> {

    must_pay(&info, NATIVE_DENOM)?;
    must_be_allowed_collection(deps.as_ref(), msg.collection_params.code_id)?;

    let params = SUDO_PARAMS.load(deps.storage)?;
    must_not_be_frozen(&params)?;

    let mut res = Response::new();
    // TODO: Creation fee -> was not discussed -> should there be one?
    checked_fair_burn(&info, params.creation_fee.amount.u128(), None, &mut res)?;

    // All checks/validations for the minter's instantiate params are done within the `new_validated` method
    msg.init_msg = OpenEditionMinterInitMsgExtension::new_validated(
        msg.init_msg.clone(),
        env,
        deps.as_ref(),
        &params
    )?;

    if NATIVE_DENOM != msg.init_msg.mint_price.denom {
        return Err(ContractError::BaseError(BaseContractError::InvalidDenom {}));
    }

    if params.min_mint_price.amount > msg.init_msg.mint_price.amount {
        return Err(ContractError::InsufficientMintPrice {
            expected: params.min_mint_price.amount.u128(),
            got: msg.init_msg.mint_price.amount.into(),
        });
    }

    let wasm_msg = WasmMsg::Instantiate {
        admin: Some(info.sender.to_string()),
        code_id: params.code_id,
        msg: to_binary(&msg)?,
        funds: vec![],
        label: format!("OpenEditionMinter-{}", msg.collection_params.name.trim()),
    };

    Ok(res
        .add_attribute("action", "create_minter")
        .add_message(wasm_msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    match msg {
        SudoMsg::UpdateParams(params_msg) => sudo_update_params(deps, env, *params_msg),
    }
}

/// Only governance can update contract params
pub fn sudo_update_params(
    deps: DepsMut,
    _env: Env,
    param_msg: OpenEditionUpdateParamsMsg,
) -> Result<Response, ContractError> {
    let mut params = SUDO_PARAMS.load(deps.storage)?;

    update_params(&mut params, param_msg.clone())?;

    params.extension.dev_fee_bps = param_msg
        .extension
        .dev_fee_bps
        .unwrap_or(params.extension.dev_fee_bps);

    params.extension.dev_fee_address = param_msg
        .extension
        .dev_fee_address
        .unwrap_or(params.extension.dev_fee_address);

    params.extension.airdrop_mint_price = param_msg
        .extension
        .airdrop_mint_price
        .unwrap_or(params.extension.airdrop_mint_price);

    params.extension.airdrop_mint_fee_bps = param_msg
        .extension
        .airdrop_mint_fee_bps
        .unwrap_or(params.extension.airdrop_mint_fee_bps);

    params.extension.abs_max_mint_per_address = param_msg
        .extension
        .abs_max_mint_per_address
        .unwrap_or(params.extension.abs_max_mint_per_address);

    params.extension.token_id_prefix_length = param_msg
        .extension
        .token_id_prefix_length
        .unwrap_or(params.extension.token_id_prefix_length);

    SUDO_PARAMS.save(deps.storage, &params)?;

    Ok(Response::new().add_attribute("action", "sudo_update_params"))
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
