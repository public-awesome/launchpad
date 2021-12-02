#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

use cw721::ContractInfoResponse;
use cw721_base::ContractError;

use crate::msg::{
    CreatorResponse, ExecuteMsg, ExtendedQueryMsg, InstantiateMsg, QueryMsg, RoyaltyResponse,
};
use crate::state::{Extension, EXTENSION};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg721";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type Sg721Contract<'a> = cw721_base::Cw721Contract<'a, Extension, Empty>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let info = ContractInfoResponse {
        name: msg.name,
        symbol: msg.symbol,
    };
    Sg721Contract::default()
        .contract_info
        .save(deps.storage, &info)?;
    let minter = deps.api.addr_validate(&msg.minter)?;
    Sg721Contract::default()
        .minter
        .save(deps.storage, &minter)?;

    EXTENSION.save(deps.storage, &msg.extension)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    Sg721Contract::default().execute(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Extended(msg) => match msg {
            ExtendedQueryMsg::Creator {} => to_binary(&query_creator(deps)?),
            ExtendedQueryMsg::Royalties {} => to_binary(&query_royalties(deps)?),
        },
        QueryMsg::Base(msg) => Sg721Contract::default().query(deps, env, msg),
    }
}

fn query_creator(deps: Deps) -> StdResult<CreatorResponse> {
    let creator = EXTENSION.load(deps.storage)?.creator;
    Ok(CreatorResponse { creator })
}

fn query_royalties(deps: Deps) -> StdResult<RoyaltyResponse> {
    let royalty = EXTENSION.load(deps.storage)?.royalties;
    Ok(RoyaltyResponse { royalty })
}
