#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, ReplyOn, Response, StdResult,
    SubMsg, WasmMsg,
};
use cw0::{parse_reply_instantiate_data, ParseReplyError};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};
use sg721::msg::InstantiateMsg as SG721InstantiateMsg;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:minter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// id for sub-message reply
const INIT_COLLECTION_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::InitCollection {
            code_id,
            name,
            symbol,
            creator,
            creator_share,
        } => execute_init_collection(deps, info, code_id, name, symbol),
    }
}

pub fn execute_init_collection(
    deps: DepsMut,
    info: MessageInfo,
    code_id: u64,
    name: String,
    symbol: String,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;
    if info.sender != state.owner {
        return Err(ContractError::Unauthorized {});
    }

    let msg = WasmMsg::Instantiate {
        admin: Some(state.owner.into_string()),
        code_id,
        // TODO: where does this come from?
        funds: vec![],
        msg: to_binary(&SG721InstantiateMsg {
            name: name.to_owned(),
            symbol: symbol.to_owned(),
            minter: info.sender.to_string(),
        })?,
        label: format!("{}-{}-{}", symbol, name, code_id),
    };

    Ok(Response::new()
        .add_attribute("method", "init_collection")
        .add_submessage(SubMsg::reply_on_success(msg, INIT_COLLECTION_ID)))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    if reply.id != INIT_COLLECTION_ID {
        return Err(ContractError::UnknownReplyId { id: reply.id });
    }

    let contract_address = match parse_reply_instantiate_data(reply) {
        Ok(res) => res.contract_address,
        Err(_) => return Err(ContractError::InvalidReplyData {}),
    };
    // TODO: save contract address

    Ok(Response::default().add_attribute("contract_address", contract_address))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::CollectionsForCreator { creator } => unimplemented!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::msg::Creator;
    use cosmwasm_std::testing::mock_dependencies_with_balance;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, Addr};

    fn setup_contract(deps: DepsMut) {
        let msg = InstantiateMsg {};
        let info = mock_info("creator", &[]);
        let res = instantiate(deps, mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
    }

    #[test]
    fn exec_init_collection() {
        let mut deps = mock_dependencies();
        let creator = String::from("creator");
        setup_contract(deps.as_mut());

        let info = mock_info(&creator, &[]);

        let msg = ExecuteMsg::InitCollection {
            code_id: 1,
            name: "collection name".to_string(),
            symbol: "SYM".to_string(),
            creator: Creator::Individual(Addr::unchecked(creator)),
            creator_share: 50u64,
        };

        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.messages.len(), 1);

        // TODO: assert contract address was saved in storage
    }
}
