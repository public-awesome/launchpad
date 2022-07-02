#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, StdResult, WasmMsg,
};
use cw2::set_contract_version;
use cw_utils::parse_reply_instantiate_data;

use crate::error::ContractError;
use crate::msg::{CountResponse, ExecuteMsg, InstantiateMsg, QueryMsg, Response, SubMsg, SudoMsg};
use crate::state::{Minter, State, MINTERS, STATE, SUDO_PARAMS};

use minter::msg::InstantiateMsg as VendingMinterInitMsg;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:factory";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        count: msg.count,
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("count", msg.count.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Increment {} => try_increment(deps),
        ExecuteMsg::Reset { count } => try_reset(deps, info, count),
        ExecuteMsg::CreateVendingMinter(msg) => execute_create_vending_minter(deps, env, info, msg),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    match msg {
        SudoMsg::VerifyMinter { minter } => todo!(),
        SudoMsg::BlockMinter { minter } => todo!(),
    }
}

pub fn execute_create_vending_minter(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: VendingMinterInitMsg,
) -> Result<Response, ContractError> {
    let params = SUDO_PARAMS.load(deps.storage)?.vending_minter;

    // TODO: charge collection creation fee here so its part of params

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
    let native_denom = deps.querier.query_bonded_denom()?;
    if native_denom != msg.unit_price.denom {
        return Err(ContractError::InvalidDenom {
            expected: native_denom,
            got: msg.unit_price.denom,
        });
    }

    // Check that the price is greater than the minimum
    if params.min_mint_price > msg.unit_price.amount.into() {
        return Err(ContractError::InsufficientMintPrice {
            expected: params.min_mint_price,
            got: msg.unit_price.amount.into(),
        });
    }

    let wasm_msg = WasmMsg::Instantiate {
        admin: Some(env.contract.address.to_string()),
        code_id: msg.sg721_code_id,
        msg: to_binary(&msg)?,
        funds: info.funds,
        label: msg.name,
    };
    let submsg = SubMsg::reply_on_success(wasm_msg, msg.sg721_code_id);

    Ok(Response::new()
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

pub fn try_increment(deps: DepsMut) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.count += 1;
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "try_increment"))
}

pub fn try_reset(deps: DepsMut, info: MessageInfo, count: i32) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        state.count = count;
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "reset"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => to_binary(&query_count(deps)?),
    }
}

fn query_count(deps: Deps) -> StdResult<CountResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(CountResponse { count: state.count })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(17, value.count);
    }

    #[test]
    fn increment() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Increment {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(18, value.count);
    }

    #[test]
    fn reset() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let unauth_info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // only the original creator can reset the counter
        let auth_info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        // should now be 5
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(5, value.count);
    }
}
