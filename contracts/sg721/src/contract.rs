#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Coin, Deps, DepsMut, Empty, Env, MessageInfo, StdResult};
use cw2::set_contract_version;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sg_std::fees::burn_and_distribute_fee;
use sg_std::StargazeMsgWrapper;

use crate::ContractError;
use cw721::ContractInfoResponse;
use cw721_base::ContractError as BaseError;
use url::Url;

use crate::msg::{
    ContractUriResponse, CreatorResponse, ExecuteMsg, InstantiateMsg, QueryMsg, RoyaltyResponse,
};
use crate::state::CONFIG;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg-721";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const CREATION_FEE: u128 = 1_000_000_000;

type Response = cosmwasm_std::Response<StargazeMsgWrapper>;
pub type Sg721Contract<'a> = cw721_base::Cw721Contract<'a, Empty, StargazeMsgWrapper>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MsgFundCommunityPool {
    pub amount: Vec<Coin>,
    pub depositor: String,
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let fee_msgs = burn_and_distribute_fee(env, &info, CREATION_FEE)?;

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

    if let Some(ref config) = msg.config {
        if let Some(ref royalty) = config.royalties {
            royalty.is_valid()?;
        }
        if let Some(ref contract_uri) = config.contract_uri {
            // Check that base_token_uri is a valid IPFS uri
            let parsed_contract_uri =
                Url::parse(contract_uri).or(Err(ContractError::InvalidContractUri {}))?;
            if parsed_contract_uri.scheme() != "ipfs" {
                return Err(ContractError::InvalidContractUri {});
            }
        }
        CONFIG.save(deps.storage, config)?;
    }

    Ok(Response::default()
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION)
        .add_messages(fee_msgs))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, BaseError> {
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
    let contract_uri = CONFIG.load(deps.storage)?.contract_uri;
    Ok(ContractUriResponse { contract_uri })
}

fn query_creator(deps: Deps) -> StdResult<CreatorResponse> {
    let creator = CONFIG.load(deps.storage)?.creator;
    Ok(CreatorResponse { creator })
}

fn query_royalties(deps: Deps) -> StdResult<RoyaltyResponse> {
    let royalty = CONFIG.load(deps.storage)?.royalties;
    Ok(RoyaltyResponse { royalty })
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::state::Config;
    use crate::state::RoyaltyInfo;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, Addr, Decimal};
    use sg_std::NATIVE_DENOM;

    #[test]
    fn proper_initialization_no_royalties() {
        let mut deps = mock_dependencies();
        let creator = String::from("creator");
        let collection = String::from("collection0");

        let msg = InstantiateMsg {
            name: collection,
            symbol: String::from("BOBO"),
            minter: String::from("minter"),
            config: Some(Config {
                contract_uri: Some(String::from("ipfs://bafyreibvxty5gjyeedk7or7tahyrzgbrwjkolpairjap3bmegvcjdipt74.ipfs.dweb.link/metadata.json")),
                creator: Some(Addr::unchecked(creator)),
                royalties: None,
            }),
        };
        let info = mock_info("creator", &coins(CREATION_FEE, NATIVE_DENOM));

        // make sure instantiate has the burn messages
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(2, res.messages.len());

        // it worked, let's query the contract_uri
        let res = query(deps.as_ref(), mock_env(), QueryMsg::ContractUri {}).unwrap();
        let value: ContractUriResponse = from_binary(&res).unwrap();
        assert_eq!(Some("ipfs://bafyreibvxty5gjyeedk7or7tahyrzgbrwjkolpairjap3bmegvcjdipt74.ipfs.dweb.link/metadata.json".to_string()), value.contract_uri);

        // it worked, let's query the creator
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Creator {}).unwrap();

        let value: CreatorResponse = from_binary(&res).unwrap();
        assert_eq!("creator", value.creator.unwrap().to_string());

        // let's query the royalties
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Royalties {}).unwrap();
        let value: RoyaltyResponse = from_binary(&res).unwrap();
        assert_eq!(None, value.royalty);
    }

    #[test]
    fn proper_initialization_with_royalties() {
        let mut deps = mock_dependencies();
        let creator = String::from("creator");
        let collection = String::from("collection0");

        let msg = InstantiateMsg {
            name: collection,
            symbol: String::from("BOBO"),
            minter: String::from("minter"),
            config: Some(Config {
                contract_uri: Some(String::from("ipfs://bafyreibvxty5gjyeedk7or7tahyrzgbrwjkolpairjap3bmegvcjdipt74.ipfs.dweb.link/metadata.json")),
                creator: Some(Addr::unchecked(creator.clone())),
                royalties: Some(RoyaltyInfo {
                    payment_address: Addr::unchecked(creator.clone()),
                    share: Decimal::percent(10),
                }),
            }),
        };
        let info = mock_info("creator", &coins(CREATION_FEE, NATIVE_DENOM));

        // make sure instantiate has the burn messages
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(2, res.messages.len());

        // it worked, let's query the contract_uri
        let res = query(deps.as_ref(), mock_env(), QueryMsg::ContractUri {}).unwrap();
        let value: ContractUriResponse = from_binary(&res).unwrap();
        assert_eq!(Some("ipfs://bafyreibvxty5gjyeedk7or7tahyrzgbrwjkolpairjap3bmegvcjdipt74.ipfs.dweb.link/metadata.json".to_string()), value.contract_uri);

        // it worked, let's query the creator
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Creator {}).unwrap();

        let value: CreatorResponse = from_binary(&res).unwrap();
        assert_eq!("creator", value.creator.unwrap().to_string());

        // let's query the royalties
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Royalties {}).unwrap();
        let value: RoyaltyResponse = from_binary(&res).unwrap();
        assert_eq!(
            Some(RoyaltyInfo {
                payment_address: Addr::unchecked(creator),
                share: Decimal::percent(10),
            }),
            value.royalty
        );
    }
}
