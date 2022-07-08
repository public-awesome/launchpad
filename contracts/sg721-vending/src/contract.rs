use std::sync::Arc;

use url::Url;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, to_vec, Binary, ContractInfoResponse as CwContractInfoResponse,
    ContractResult, Decimal, Deps, DepsMut, Empty, Env, MessageInfo, Querier, QueryRequest,
    StdError, StdResult, SystemResult, WasmQuery,
};
use cw2::set_contract_version;
use cw721::ContractInfoResponse;
use cw721_base::ContractError as BaseError;
use cw_utils::nonpayable;

use launchpad::{ParamsResponse, QueryMsg as LaunchpadQueryMsg};
use minter::msg::{ConfigResponse, QueryMsg as MinterQueryMsg};
use sg721::{CollectionInfo, InstantiateMsg, RoyaltyInfo, RoyaltyInfoResponse};
use sg_std::{Response, StargazeMsgWrapper};

use crate::msg::{CollectionInfoResponse, ExecuteMsg, QueryMsg};
use crate::state::COLLECTION_INFO;
use crate::ContractError;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg-721";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MAX_DESCRIPTION_LENGTH: u32 = 512;

pub type Sg721Contract<'a> = cw721_base::Cw721Contract<'a, Empty, StargazeMsgWrapper>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // no funds should be sent to this contract
    nonpayable(&info)?;

    println!("minter {:?}", msg.minter);

    // TODO: move these checks into the approve() function
    // called by the minter reply

    // TODO: keep track of contract setup state
    // only allow actions when contract is approved

    // query minter to the get the factory address
    let config: ConfigResponse = deps
        .querier
        .query_wasm_smart(msg.minter.clone(), &MinterQueryMsg::Config {})?;
    let factory = config.factory;
    if factory != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // query minter contract to get minter code id
    let query = WasmQuery::ContractInfo {
        contract_addr: msg.minter.clone(),
    }
    .into();
    let res: CwContractInfoResponse = deps.querier.query(&query)?;
    let minter_id = res.code_id;

    // query factory to check if minter code id is in allowed list
    let res: ParamsResponse = deps
        .querier
        .query_wasm_smart(factory, &LaunchpadQueryMsg::Params {})?;
    if !res.params.minter_codes.iter().any(|x| x == &minter_id) {
        return Err(ContractError::Unauthorized {});
    }

    // cw721 instantiation
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

    // sg721 instantiation
    if msg.collection_info.description.len() > MAX_DESCRIPTION_LENGTH as usize {
        return Err(ContractError::DescriptionTooLong {});
    }

    let image = Url::parse(&msg.collection_info.image)?;

    if let Some(ref external_link) = msg.collection_info.external_link {
        Url::parse(external_link)?;
    }

    let royalty_info: Option<RoyaltyInfo> = match msg.collection_info.royalty_info {
        Some(royalty_info) => Some(RoyaltyInfo {
            payment_address: deps.api.addr_validate(&royalty_info.payment_address)?,
            share: share_validate(royalty_info.share)?,
        }),
        None => None,
    };

    deps.api.addr_validate(&msg.collection_info.creator)?;

    let collection_info = CollectionInfo {
        creator: msg.collection_info.creator,
        description: msg.collection_info.description,
        image: msg.collection_info.image,
        external_link: msg.collection_info.external_link,
        royalty_info,
    };

    COLLECTION_INFO.save(deps.storage, &collection_info)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION)
        .add_attribute("image", image.to_string()))
}

fn query_contract_info(
    querier: &dyn Querier,
    contract_addr: String,
) -> StdResult<CwContractInfoResponse> {
    let raw = QueryRequest::<Empty>::Wasm(WasmQuery::ContractInfo { contract_addr });
    match querier.raw_query(&to_vec(&raw).unwrap()) {
        SystemResult::Err(system_err) => Err(StdError::generic_err(format!(
            "Querier system error: {}",
            system_err
        ))),
        SystemResult::Ok(ContractResult::Err(contract_err)) => Err(StdError::generic_err(format!(
            "Querier contract error: {}",
            contract_err
        ))),
        SystemResult::Ok(ContractResult::Ok(res)) => {
            let response: CwContractInfoResponse = from_binary(&res)?;
            Ok(response)
        }
    }
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
        QueryMsg::CollectionInfo {} => to_binary(&query_config(deps)?),
        _ => Sg721Contract::default().query(deps, env, msg.into()),
    }
}

fn query_config(deps: Deps) -> StdResult<CollectionInfoResponse> {
    let info = COLLECTION_INFO.load(deps.storage)?;

    let royalty_info_res: Option<RoyaltyInfoResponse> = match info.royalty_info {
        Some(royalty_info) => Some(RoyaltyInfoResponse {
            payment_address: royalty_info.payment_address.to_string(),
            share: royalty_info.share,
        }),
        None => None,
    };

    Ok(CollectionInfoResponse {
        creator: info.creator,
        description: info.description,
        image: info.image,
        external_link: info.external_link,
        royalty_info: royalty_info_res,
    })
}

pub fn share_validate(share: Decimal) -> Result<Decimal, ContractError> {
    if share > Decimal::one() {
        return Err(ContractError::InvalidRoyalities {});
    }

    Ok(share)
}

// FIXME: fail because minter doesn't exist yet
// #[cfg(test)]
// mod tests {
//     use super::*;

//     use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
//     use cosmwasm_std::{coins, from_binary, Decimal};
//     use sg_std::NATIVE_DENOM;

//     #[test]
//     fn proper_initialization_no_royalties() {
//         let mut deps = mock_dependencies();
//         let collection = String::from("collection0");

//         let msg = InstantiateMsg {
//             name: collection,
//             symbol: String::from("BOBO"),
//             minter: String::from("minter"),
//             collection_info: CollectionInfo {
//                 creator: String::from("creator"),
//                 description: String::from("Stargaze Monkeys"),
//                 image: "https://example.com/image.png".to_string(),
//                 external_link: Some("https://example.com/external.html".to_string()),
//                 royalty_info: None,
//             },
//         };
//         let info = mock_info("creator", &[]);

//         let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
//         assert_eq!(0, res.messages.len());

//         // let's query the collection info
//         let res = query(deps.as_ref(), mock_env(), QueryMsg::CollectionInfo {}).unwrap();
//         let value: CollectionInfoResponse = from_binary(&res).unwrap();
//         assert_eq!("https://example.com/image.png", value.image);
//         assert_eq!("Stargaze Monkeys", value.description);
//         assert_eq!(
//             "https://example.com/external.html",
//             value.external_link.unwrap()
//         );
//         assert_eq!(None, value.royalty_info);
//     }

//     #[test]
//     fn proper_initialization_with_royalties() {
//         let mut deps = mock_dependencies();
//         let creator = String::from("creator");
//         let collection = String::from("collection0");

//         let msg = InstantiateMsg {
//             name: collection,
//             symbol: String::from("BOBO"),
//             minter: String::from("minter"),
//             collection_info: CollectionInfo {
//                 creator: String::from("creator"),
//                 description: String::from("Stargaze Monkeys"),
//                 image: "https://example.com/image.png".to_string(),
//                 external_link: Some("https://example.com/external.html".to_string()),
//                 royalty_info: Some(RoyaltyInfoResponse {
//                     payment_address: creator.clone(),
//                     share: Decimal::percent(10),
//                 }),
//             },
//         };
//         let info = mock_info("creator", &[]);

//         let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
//         assert_eq!(0, res.messages.len());

//         // let's query the collection info
//         let res = query(deps.as_ref(), mock_env(), QueryMsg::CollectionInfo {}).unwrap();
//         let value: CollectionInfoResponse = from_binary(&res).unwrap();
//         assert_eq!(
//             Some(RoyaltyInfoResponse {
//                 payment_address: creator,
//                 share: Decimal::percent(10),
//             }),
//             value.royalty_info
//         );
//     }
// }
