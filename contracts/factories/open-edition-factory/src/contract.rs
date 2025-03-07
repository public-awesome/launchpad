#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    WasmMsg,
};
use cw2::set_contract_version;
use semver::Version;
use sg_utils::NATIVE_DENOM;

use base_factory::contract::{
    must_be_allowed_collection, must_not_be_frozen, must_pay_exact_amount, update_params,
};
use base_factory::ContractError as BaseContractError;
use sg1::{checked_fair_burn, transfer_funds_to_launchpad_dao};
use sg2::query::{AllowedCollectionCodeIdResponse, AllowedCollectionCodeIdsResponse, Sg2QueryMsg};

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, OpenEditionMinterCreateMsg, OpenEditionMinterInitMsgExtension,
    OpenEditionUpdateParamsMsg, ParamsResponse, SudoMsg,
};
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
    let params = msg.params;

    SUDO_PARAMS.save(deps.storage, &params)?;

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
    let params = SUDO_PARAMS.load(deps.storage)?;

    must_pay_exact_amount(&params, &info, &params.creation_fee.denom)?;

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

    msg.init_msg = OpenEditionMinterInitMsgExtension::validate(
        msg.init_msg.clone(),
        env,
        deps.as_ref(),
        &params,
    )?;

    ensure!(
        params.min_mint_price.denom == msg.init_msg.mint_price.denom,
        BaseContractError::InvalidDenom {}
    );

    ensure!(
        params.min_mint_price.amount <= msg.init_msg.mint_price.amount,
        ContractError::InsufficientMintPrice {
            expected: params.min_mint_price.amount.u128(),
            got: msg.init_msg.mint_price.amount.into(),
        }
    );

    if msg.init_msg.num_tokens.is_none() {
        ensure!(
            !msg.init_msg.mint_price.amount.is_zero(),
            ContractError::NoTokenLimitWithZeroMintPrice {}
        );
    }

    if params.extension.airdrop_mint_price.amount.is_zero() {
        ensure!(
            msg.init_msg.num_tokens.is_some(),
            ContractError::NoTokenLimitWithZeroAirdropPrice {}
        );
    }

    let wasm_msg = WasmMsg::Instantiate {
        admin: Some(info.sender.to_string()),
        code_id: params.code_id,
        msg: to_json_binary(&msg)?,
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

    params.extension.max_token_limit = param_msg
        .extension
        .max_token_limit
        .unwrap_or(params.extension.max_token_limit);

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

    params.extension.max_per_address_limit = param_msg
        .extension
        .max_per_address_limit
        .unwrap_or(params.extension.max_per_address_limit);

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
    msg: Option<OpenEditionUpdateParamsMsg>,
) -> Result<Response, base_factory::ContractError> {
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

        params.extension.dev_fee_address = msg
            .extension
            .dev_fee_address
            .unwrap_or(params.extension.dev_fee_address);

        params.extension.airdrop_mint_price = msg
            .extension
            .airdrop_mint_price
            .unwrap_or(params.extension.airdrop_mint_price);

        params.extension.airdrop_mint_fee_bps = msg
            .extension
            .airdrop_mint_fee_bps
            .unwrap_or(params.extension.airdrop_mint_fee_bps);

        params.extension.max_per_address_limit = msg
            .extension
            .max_per_address_limit
            .unwrap_or(params.extension.max_per_address_limit);

        SUDO_PARAMS.save(deps.storage, &params)?;
    }

    Ok(Response::new().add_attribute("action", "migrate"))
}
