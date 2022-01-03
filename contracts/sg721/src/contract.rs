#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

use cw721::ContractInfoResponse;
use cw721_base::ContractError;

use crate::msg::{
    ContractUriResponse, CreatorResponse, ExecuteMsg, InstantiateMsg, QueryMsg, RoyaltyResponse,
};
use crate::state::{State, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg721";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type Sg721Contract<'a> = cw721_base::Cw721Contract<'a, Empty, Empty>;

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

    STATE.save(deps.storage, &msg.state)?;

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
        QueryMsg::ContractUri {} => to_binary(&query_contract_uri(deps)?),
        QueryMsg::Creator {} => to_binary(&query_creator(deps)?),
        QueryMsg::Royalties {} => to_binary(&query_royalties(deps)?),
        _ => Sg721Contract::default().query(deps, env, msg.into()),
    }
}

fn query_contract_uri(deps: Deps) -> StdResult<ContractUriResponse> {
    let contract_uri = STATE.load(deps.storage)?.contract_uri;
    Ok(ContractUriResponse { contract_uri })
}

fn query_creator(deps: Deps) -> StdResult<CreatorResponse> {
    let creator = STATE.load(deps.storage)?.royalties.creator;
    Ok(CreatorResponse { creator })
}

fn query_royalties(deps: Deps) -> StdResult<RoyaltyResponse> {
    let royalty = STATE.load(deps.storage)?.royalties;
    Ok(RoyaltyResponse { royalty })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, Addr, Decimal};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        let creator = String::from("creator");
        let collection = String::from("collection0");

        let msg = InstantiateMsg {
            name: collection,
            symbol: String::from("BOBO"),
            minter: String::from("minter"),
            state: State {
                contract_uri: String::from("https://bafyreibvxty5gjyeedk7or7tahyrzgbrwjkolpairjap3bmegvcjdipt74.ipfs.dweb.link/metadata.json"),
                royalties: crate::state::RoyaltyInfo { creator: Addr::unchecked(creator.clone()), creator_payment_address: None, owner_payment_address: None, creator_share: Decimal::percent(50), owner_share: Decimal::percent(50) },
            },
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        // make sure instantiate doesn't send any messages
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the contract_uri
        let res = query(deps.as_ref(), mock_env(), QueryMsg::ContractUri {}).unwrap();
        let value: ContractUriResponse = from_binary(&res).unwrap();
        assert_eq!("https://bafyreibvxty5gjyeedk7or7tahyrzgbrwjkolpairjap3bmegvcjdipt74.ipfs.dweb.link/metadata.json", value.contract_uri);

        // it worked, let's query the creator
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Creator {}).unwrap();

        let value: CreatorResponse = from_binary(&res).unwrap();
        assert_eq!("creator", value.creator.to_string());

        // let's query the royalties
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Royalties {}).unwrap();
        let value: RoyaltyResponse = from_binary(&res).unwrap();
        assert_eq!(creator.clone(), value.royalty.creator);
    }
}
