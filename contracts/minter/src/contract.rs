#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Api, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Order, Reply, Response,
    StdResult, SubMsg, WasmMsg, WasmQuery,
};
use cw0::parse_reply_instantiate_data;
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use sg721::state::CreatorInfo;

use crate::error::ContractError;
use crate::msg::{CollectionsResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, COLLECTIONS, STATE};
use sg721::msg::{ExtendedQueryMsg, InstantiateMsg as SG721InstantiateMsg};

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
    _msg: InstantiateMsg,
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
    env: Env,
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
        } => execute_init_collection(
            deps,
            info,
            env,
            code_id,
            name,
            symbol,
            creator,
            creator_share,
        ),
    }
}

pub fn execute_init_collection(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    code_id: u64,
    name: String,
    symbol: String,
    creator: Addr,
    creator_share: u64,
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
            creator_info: CreatorInfo {
                creator,
                creator_share: creator_share,
            },
        })?,
        label: format!("{}-{}-{}", symbol, name, code_id),
    };

    Ok(Response::new()
        .add_attribute("method", "init_collection")
        .add_submessage(SubMsg::reply_on_success(msg, INIT_COLLECTION_ID)))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    if reply.id != INIT_COLLECTION_ID {
        return Err(ContractError::UnknownReplyId { id: reply.id });
    }
    println!("in reply!");

    // Now a new sg721 contract should have been instantiated. We can query
    // it to get the creator, and save the creator <> contract association.

    // get the contract address from the sub-message reply
    let contract_address = match parse_reply_instantiate_data(reply) {
        Ok(res) => res.contract_address,
        Err(_) => return Err(ContractError::InvalidReplyData {}),
    };
    let contract_addr = deps.api.addr_validate(contract_address.as_str())?;

    // query the newly created contract for the creator
    let msg = ExtendedQueryMsg::Creator {};
    let query = WasmQuery::Smart {
        contract_addr: contract_address.to_string(),
        msg: to_binary(&msg)?,
    };
    let creator_info: CreatorInfo = deps
        .querier
        .query_wasm_smart(contract_address.to_string(), &query)
        .unwrap();

    // save creator <> contract in storage
    COLLECTIONS.save(
        deps.storage,
        (&creator_info.creator, &contract_addr),
        &Empty {},
    )?;

    Ok(Response::default().add_attribute("contract_address", contract_address))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Collections { creator } => to_binary(&query_collections(deps, creator)?),
    }
}

fn query_collections(deps: Deps, creator: Addr) -> StdResult<CollectionsResponse> {
    // TODO: would an IndexedMap make more sense since it provides helpers to iterate?
    // iterate collections by creator
    let collections = COLLECTIONS
        .prefix(&creator)
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| item.map(|k| String::from_utf8(k.0).unwrap()))
        .map(|s| Addr::unchecked(s.unwrap()))
        .collect();

    Ok(CollectionsResponse { collections })
}

#[cfg(test)]
mod tests {
    use super::*;
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
            creator: Addr::unchecked(creator),
            creator_share: 50u64,
        };

        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.messages.len(), 1);

        // TODO: fake the call to reply

        let collections = query_collections(deps.as_ref(), Addr::unchecked("creator")).unwrap();
        assert_eq!(collections.collections.len(), 1);
    }
}
