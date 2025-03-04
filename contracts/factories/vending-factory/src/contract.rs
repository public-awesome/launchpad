use base_factory::contract::{must_be_allowed_collection, must_not_be_frozen, update_params};
use base_factory::ContractError as BaseContractError;
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
use sg2::query::{AllowedCollectionCodeIdResponse, AllowedCollectionCodeIdsResponse, Sg2QueryMsg};
use sg_utils::NATIVE_DENOM;

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, ParamsResponse, SudoMsg, VendingMinterCreateMsg,
    VendingUpdateParamsMsg,
};
use crate::state::SUDO_PARAMS;

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
        ExecuteMsg::CreateMinter(msg) => execute_create_minter(deps, env, info, msg),
    }
}

pub fn execute_create_minter(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: VendingMinterCreateMsg,
) -> Result<Response, ContractError> {
    let params = SUDO_PARAMS.load(deps.storage)?;
    must_pay(&info, &params.creation_fee.denom)?;
    must_be_allowed_collection(deps.as_ref(), msg.collection_params.code_id)?;

    must_not_be_frozen(&params)?;

    let mut res = Response::new();
    if params.creation_fee.denom == NATIVE_DENOM {
        checked_fair_burn(
            &info,
            &env,
            params.creation_fee.amount.u128(),
            None,
            &mut res,
        )?;
    } else {
        transfer_funds_to_launchpad_dao(
            &info,
            params.creation_fee.amount.u128(),
            &params.creation_fee.denom,
            &mut res,
        )?;
    }

    // Check the number of tokens is more than zero and less than the max limit
    if msg.init_msg.num_tokens == 0 || msg.init_msg.num_tokens > params.extension.max_token_limit {
        return Err(ContractError::InvalidNumTokens {
            min: 1,
            max: params.extension.max_token_limit,
        });
    }

    // Check per address limit is valid
    if msg.init_msg.per_address_limit == 0
        || msg.init_msg.per_address_limit > params.extension.max_per_address_limit
    {
        return Err(ContractError::InvalidPerAddressLimit {
            max: params.extension.max_per_address_limit,
            min: 1,
            got: msg.init_msg.per_address_limit,
        });
    }

    ensure!(
        params.min_mint_price.denom == msg.init_msg.mint_price.denom,
        ContractError::DenomMismatch {}
    );

    if params.min_mint_price.amount > msg.init_msg.mint_price.amount {
        return Err(ContractError::InsufficientMintPrice {
            expected: params.min_mint_price.amount.u128(),
            got: msg.init_msg.mint_price.amount.into(),
        });
    }

    let wasm_msg = WasmMsg::Instantiate {
        admin: Some(info.sender.to_string()),
        code_id: params.code_id,
        msg: to_json_binary(&msg)?,
        funds: vec![],
        label: format!("VendingMinter-{}", msg.collection_params.name.trim()),
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
    param_msg: VendingUpdateParamsMsg,
) -> Result<Response, ContractError> {
    let mut params = SUDO_PARAMS.load(deps.storage)?;

    update_params(&mut params, param_msg.clone())?;

    params.extension.max_token_limit = param_msg
        .extension
        .max_token_limit
        .unwrap_or(params.extension.max_token_limit);
    params.extension.max_per_address_limit = param_msg
        .extension
        .max_per_address_limit
        .unwrap_or(params.extension.max_per_address_limit);

    if let Some(airdrop_mint_price) = param_msg.extension.airdrop_mint_price {
        ensure_eq!(
            &airdrop_mint_price.denom,
            &NATIVE_DENOM,
            ContractError::BaseError(BaseContractError::InvalidDenom {})
        );
        params.extension.airdrop_mint_price = airdrop_mint_price;
    }

    params.extension.airdrop_mint_fee_bps = param_msg
        .extension
        .airdrop_mint_fee_bps
        .unwrap_or(params.extension.airdrop_mint_fee_bps);

    if let Some(shuffle_fee) = param_msg.extension.shuffle_fee {
        ensure_eq!(
            &shuffle_fee.denom,
            &NATIVE_DENOM,
            ContractError::BaseError(BaseContractError::InvalidDenom {})
        );
        params.extension.shuffle_fee = shuffle_fee;
    }

    SUDO_PARAMS.save(deps.storage, &params)?;

    Ok(Response::new().add_attribute("action", "sudo_update_params"))
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
    msg: Option<VendingUpdateParamsMsg>,
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

        update_params(&mut params, msg.clone())?;

        params.extension.max_token_limit = msg
            .extension
            .max_token_limit
            .unwrap_or(params.extension.max_token_limit);
        params.extension.max_per_address_limit = msg
            .extension
            .max_per_address_limit
            .unwrap_or(params.extension.max_per_address_limit);

        if let Some(airdrop_mint_price) = msg.extension.airdrop_mint_price {
            ensure_eq!(
                &airdrop_mint_price.denom,
                &NATIVE_DENOM,
                ContractError::BaseError(BaseContractError::InvalidDenom {})
            );
            params.extension.airdrop_mint_price = airdrop_mint_price;
        }

        params.extension.airdrop_mint_fee_bps = msg
            .extension
            .airdrop_mint_fee_bps
            .unwrap_or(params.extension.airdrop_mint_fee_bps);

        if let Some(shuffle_fee) = msg.extension.shuffle_fee {
            ensure_eq!(
                &shuffle_fee.denom,
                &NATIVE_DENOM,
                ContractError::BaseError(BaseContractError::InvalidDenom {})
            );
            params.extension.shuffle_fee = shuffle_fee;
        }

        SUDO_PARAMS.save(deps.storage, &params)?;
    }
    Ok(Response::new().add_attribute("action", "migrate"))
}
