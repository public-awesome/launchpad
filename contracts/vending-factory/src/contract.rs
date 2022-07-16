#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure_eq, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, StdResult, WasmMsg,
};
use cw2::set_contract_version;
use cw_utils::{must_pay, parse_reply_instantiate_data};
use sg1::checked_fair_burn;
use sg_std::NATIVE_DENOM;
use vending::{ExecuteMsg, ParamsResponse, VendingMinterCreateMsg};

use crate::error::ContractError;
use crate::msg::{InstantiateMsg, QueryMsg, Response, SubMsg, SudoMsg, UpdateParamsMsg};
use crate::state::{Minter, MINTERS, SUDO_PARAMS};

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
        ExecuteMsg::CreateVendingMinter(msg) => execute_create_vending_minter(deps, env, info, msg),
    }
}

pub fn execute_create_vending_minter(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: VendingMinterCreateMsg,
) -> Result<Response, ContractError> {
    // TODO: why doesn't this work?
    // must_pay(&info, &deps.querier.query_bonded_denom()?)?;
    must_pay(&info, NATIVE_DENOM)?;

    let params = SUDO_PARAMS.load(deps.storage)?;

    let mut res = Response::new();
    checked_fair_burn(&info, params.creation_fee.amount.u128(), None, &mut res)?;

    // Check the number of tokens is more than zero and less than the max limit
    if msg.init_msg.num_tokens == 0 || msg.init_msg.num_tokens > params.max_token_limit {
        return Err(ContractError::InvalidNumTokens {
            min: 1,
            max: params.max_token_limit,
        });
    }

    // Check per address limit is valid
    if msg.init_msg.per_address_limit == 0
        || msg.init_msg.per_address_limit > params.max_per_address_limit
    {
        return Err(ContractError::InvalidPerAddressLimit {
            max: params.max_per_address_limit,
            min: 1,
            got: msg.init_msg.per_address_limit,
        });
    }

    // Check that the price is in the correct denom ('ustars')
    // let native_denom = deps.querier.query_bonded_denom()?;
    let native_denom = NATIVE_DENOM;
    if native_denom != msg.init_msg.unit_price.denom {
        return Err(ContractError::InvalidDenom {});
    }

    // Check that the price is greater than the minimum
    if params.min_mint_price.amount > msg.init_msg.unit_price.amount {
        return Err(ContractError::InsufficientMintPrice {
            expected: params.min_mint_price.amount.u128(),
            got: msg.init_msg.unit_price.amount.into(),
        });
    }

    let wasm_msg = WasmMsg::Instantiate {
        admin: Some(info.sender.to_string()),
        code_id: params.code_id,
        msg: to_binary(&msg)?,
        funds: vec![],
        label: format!("VendingMinter-{}", msg.collection_params.name),
    };
    let submsg = SubMsg::reply_on_success(wasm_msg, params.code_id);

    Ok(res
        .add_attribute("action", "create_minter")
        .add_submessage(submsg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    match msg {
        SudoMsg::UpdateParams(params_msg) => sudo_update_params(deps, env, params_msg),
        SudoMsg::UpdateMinterStatus {
            minter,
            verified,
            blocked,
        } => sudo_update_minter_status(deps, env, minter, verified, blocked),
    }
}

/// Only governance can update contract params
pub fn sudo_update_minter_status(
    deps: DepsMut,
    _env: Env,
    minter: String,
    verified: bool,
    blocked: bool,
) -> Result<Response, ContractError> {
    let minter_addr = deps.api.addr_validate(&minter)?;

    let _: StdResult<Minter> = MINTERS.update(deps.storage, &minter_addr, |m| match m {
        None => Ok(Minter { verified, blocked }),
        Some(mut m) => {
            m.verified = verified;
            m.blocked = blocked;
            Ok(m)
        }
    });

    Ok(Response::new().add_attribute("action", "sudo_update_minter_status"))
}

/// Only governance can update contract params
pub fn sudo_update_params(
    deps: DepsMut,
    _env: Env,
    param_msg: UpdateParamsMsg,
) -> Result<Response, ContractError> {
    let mut params = SUDO_PARAMS.load(deps.storage)?;
    let native_denom = deps.querier.query_bonded_denom()?;

    params.code_id = param_msg.code_id.unwrap_or(params.code_id);

    if let Some(creation_fee) = param_msg.creation_fee {
        ensure_eq!(
            &creation_fee.denom,
            &native_denom,
            ContractError::InvalidDenom {}
        );
        params.creation_fee = creation_fee;
    }

    params.max_token_limit = param_msg.max_token_limit.unwrap_or(params.max_token_limit);
    params.max_per_address_limit = param_msg
        .max_per_address_limit
        .unwrap_or(params.max_per_address_limit);

    if let Some(min_mint_price) = param_msg.min_mint_price {
        ensure_eq!(
            &min_mint_price.denom,
            &native_denom,
            ContractError::InvalidDenom {}
        );
        params.min_mint_price = min_mint_price;
    }

    if let Some(airdrop_mint_price) = param_msg.airdrop_mint_price {
        ensure_eq!(
            &airdrop_mint_price.denom,
            &native_denom,
            ContractError::InvalidDenom {}
        );
        params.airdrop_mint_price = airdrop_mint_price;
    }

    params.mint_fee_bps = param_msg.mint_fee_bps.unwrap_or(params.mint_fee_bps);
    params.airdrop_mint_fee_bps = param_msg
        .airdrop_mint_fee_bps
        .unwrap_or(params.airdrop_mint_fee_bps);

    if let Some(shuffle_fee) = param_msg.shuffle_fee {
        ensure_eq!(
            &shuffle_fee.denom,
            &native_denom,
            ContractError::InvalidDenom {}
        );
        params.extension.shuffle_fee = shuffle_fee;
    }

    SUDO_PARAMS.save(deps.storage, &params)?;

    Ok(Response::new().add_attribute("action", "sudo_update_params"))
}

// Reply callback triggered from cw721 contract instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    let code_id = msg.id;
    let reply = parse_reply_instantiate_data(msg);

    match reply {
        Ok(res) => {
            let minter = Minter {
                verified: false,
                blocked: false,
            };
            // TODO: save factory contract address in minter config

            MINTERS.save(
                deps.storage,
                &Addr::unchecked(res.contract_address),
                &minter,
            )?;
            Ok(Response::default().add_attribute("action", "instantiate_minter_reply"))
        }
        Err(_) => Err(ContractError::InstantiateMinterError {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Params {} => to_binary(&query_params(deps)?),
    }
}

fn query_params(deps: Deps) -> StdResult<ParamsResponse> {
    let params = SUDO_PARAMS.load(deps.storage)?;
    Ok(ParamsResponse { params })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::coins;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use vending::tests::mock_params;

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            params: mock_params(),
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // // it worked, let's query the state
        // let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        // let value: CountResponse = from_binary(&res).unwrap();
        // assert_eq!(17, value.count);
    }
}
