#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, StdResult, WasmMsg,
};
use cw2::set_contract_version;
use cw_utils::{must_pay, parse_reply_instantiate_data};
use sg1::checked_fair_burn;
use sg_std::NATIVE_DENOM;
use vending::{ExecuteMsg, ParamsResponse, VendingMinterInitMsg};

use crate::error::ContractError;
use crate::msg::{InstantiateMsg, QueryMsg, Response, SubMsg, SudoMsg};
use crate::state::{Minter, MINTERS, SUDO_PARAMS};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:factory";
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

    // TODO: validate params

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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(_deps: DepsMut, _env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    match msg {
        SudoMsg::VerifyMinter { minter } => todo!(),
        SudoMsg::BlockMinter { minter } => todo!(),
    }
}

pub fn execute_create_vending_minter(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mut msg: VendingMinterInitMsg,
) -> Result<Response, ContractError> {
    // TODO: why doesn't this work?
    // must_pay(&info, &deps.querier.query_bonded_denom()?)?;
    must_pay(&info, NATIVE_DENOM)?;

    let params = SUDO_PARAMS.load(deps.storage)?.vending_minter;

    let mut res = Response::new();
    checked_fair_burn(&info, params.creation_fee.u128(), None, &mut res)?;

    msg.factory = env.contract.address.to_string();

    // Check the number of tokens is more than zero and less than the max limit
    if msg.num_tokens == 0 || msg.num_tokens > params.max_token_limit {
        return Err(ContractError::InvalidNumTokens {
            min: 1,
            max: params.max_token_limit,
        });
    }

    // Check per address limit is valid
    if msg.per_address_limit == 0 || msg.per_address_limit > params.max_per_address_limit {
        return Err(ContractError::InvalidPerAddressLimit {
            max: params.max_per_address_limit,
            min: 1,
            got: msg.per_address_limit,
        });
    }

    // Check that the price is in the correct denom ('ustars')
    // let native_denom = deps.querier.query_bonded_denom()?;
    let native_denom = NATIVE_DENOM;
    if native_denom != msg.unit_price.denom {
        return Err(ContractError::InvalidDenom {
            expected: native_denom.to_string(),
            got: msg.unit_price.denom,
        });
    }

    // Check that the price is greater than the minimum
    if params.min_mint_price > msg.unit_price.amount {
        return Err(ContractError::InsufficientMintPrice {
            expected: params.min_mint_price.u128(),
            got: msg.unit_price.amount.into(),
        });
    }

    let wasm_msg = WasmMsg::Instantiate {
        admin: Some(info.sender.to_string()),
        code_id: params.code_id,
        msg: to_binary(&msg)?,
        funds: vec![],
        label: format!("VendingMinter-{}", msg.name),
    };
    let submsg = SubMsg::reply_on_success(wasm_msg, msg.sg721_code_id);

    Ok(res
        .add_attribute("action", "create_minter")
        .add_submessage(submsg))
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
                (code_id, &Addr::unchecked(res.contract_address)),
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
    use vending::{SudoParams, VendingMinterParams};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let sudo_params = SudoParams {
            minter_codes: vec![1, 2, 3],
            vending_minter: VendingMinterParams::default(),
        };

        let msg = InstantiateMsg {
            params: sudo_params,
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
