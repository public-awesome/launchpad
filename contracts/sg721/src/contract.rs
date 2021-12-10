#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

use cw721::ContractInfoResponse;
use cw721_base::ContractError;

use crate::msg::{CreatorResponse, ExecuteMsg, InstantiateMsg, QueryMsg, RoyaltyResponse};
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
        QueryMsg::OwnerOf {
            token_id,
            include_expired,
        } => todo!(),
        QueryMsg::ApprovedForAll {
            owner,
            include_expired,
            start_after,
            limit,
        } => todo!(),
        QueryMsg::NumTokens {} => todo!(),
        QueryMsg::ContractInfo {} => todo!(),
        QueryMsg::NftInfo { token_id } => todo!(),
        QueryMsg::AllNftInfo {
            token_id,
            include_expired,
        } => todo!(),
        QueryMsg::Tokens {
            owner,
            start_after,
            limit,
        } => todo!(),
        QueryMsg::AllTokens { start_after, limit } => todo!(),
        QueryMsg::Minter {} => todo!(),
        QueryMsg::Creator {} => to_binary(&query_creator(deps)?),
        QueryMsg::Royalties {} => to_binary(&query_royalties(deps)?),
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

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, Addr};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        let creator = String::from("creator");
        let collection = String::from("collection0");

        let msg = InstantiateMsg {
            name: collection,
            symbol: String::from("BOBO"),
            minter: String::from("minter"),
            extension: Extension {
                creator: Addr::unchecked(creator),
                royalties: None,
            },
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        // make sure instantiate doesn't send any messages
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the creator
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Creator {}).unwrap();
        let value: CreatorResponse = from_binary(&res).unwrap();
        assert_eq!("creator", value.creator.to_string());

        // let's query the royalties
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Royalties {}).unwrap();
        let value: RoyaltyResponse = from_binary(&res).unwrap();
        assert_eq!(None, value.royalty);
    }
}
