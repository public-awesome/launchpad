use cw721_base::state::TokenInfo;
use cw721_base::Extension;
use url::Url;

use cosmwasm_std::{
    from_binary, to_binary, to_vec, Binary, ContractInfoResponse as CwContractInfoResponse,
    ContractResult, Decimal, Deps, DepsMut, Empty, Env, Event, MessageInfo, Querier, QueryRequest,
    StdError, StdResult, SystemResult, WasmQuery,
};
use cw2::set_contract_version;
use cw721::{ContractInfoResponse, Cw721ReceiveMsg};
use cw_utils::{nonpayable, Expiration};

use launchpad::{ParamsResponse, QueryMsg as LaunchpadQueryMsg};
use minter::msg::{ConfigResponse, QueryMsg as MinterQueryMsg};
use sg721::{CollectionInfo, InstantiateMsg, MintMsg, RoyaltyInfo, RoyaltyInfoResponse};
use sg_std::Response;

use crate::msg::{CollectionInfoResponse, QueryMsg};
use crate::state::COLLECTION_INFO;
use crate::{ContractError, Sg721Contract};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg-721";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MAX_DESCRIPTION_LENGTH: u32 = 512;

pub fn _instantiate(
    contract: Sg721Contract,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // no funds should be sent to this contract
    nonpayable(&info)?;

    println!("minter {:?}", msg.minter);

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

// TODO: make sure this cannot be called from the outside
pub fn ready(
    contract: Sg721Contract,
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let minter = Sg721Contract::default().minter.load(deps.storage)?;
    if minter != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // query minter to the get the factory address
    let config: ConfigResponse = deps
        .querier
        .query_wasm_smart(minter.clone(), &MinterQueryMsg::Config {})?;
    let factory = config.factory;

    // query minter contract to get minter code id
    let query = WasmQuery::ContractInfo {
        contract_addr: minter.to_string(),
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

    Ok(Response::new())
}

pub fn approve(
    contract: Sg721Contract,
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    spender: String,
    token_id: String,
    expires: Option<Expiration>,
) -> Result<Response, ContractError> {
    contract._update_approvals(deps, &env, &info, &spender, &token_id, true, expires)?;

    let event = Event::new("approve")
        .add_attribute("sender", info.sender)
        .add_attribute("spender", spender)
        .add_attribute("token_id", token_id);
    let res = Response::new().add_event(event);

    Ok(res)
}

pub fn approve_all(
    contract: Sg721Contract,
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    operator: String,
    expires: Option<Expiration>,
) -> Result<Response, ContractError> {
    // reject expired data as invalid
    let expires = expires.unwrap_or_default();
    if expires.is_expired(&env.block) {
        return Err(ContractError::Expired {});
    }

    // set the operator for us
    let operator_addr = deps.api.addr_validate(&operator)?;
    contract
        .operators
        .save(deps.storage, (&info.sender, &operator_addr), &expires)?;

    let event = Event::new("approve_all")
        .add_attribute("sender", info.sender)
        .add_attribute("operator", operator);
    let res = Response::new().add_event(event);

    Ok(res)
}

pub fn burn(
    contract: Sg721Contract,
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {
    let token = contract.tokens.load(deps.storage, &token_id)?;
    contract.check_can_send(deps.as_ref(), &env, &info, &token)?;

    contract.tokens.remove(deps.storage, &token_id)?;
    contract.decrement_tokens(deps.storage)?;

    let event = Event::new("burn")
        .add_attribute("sender", info.sender)
        .add_attribute("token_id", token_id);
    let res = Response::new().add_event(event);

    Ok(res)
}

pub fn revoke(
    contract: Sg721Contract,
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    spender: String,
    token_id: String,
) -> Result<Response, ContractError> {
    contract._update_approvals(deps, &env, &info, &spender, &token_id, false, None)?;

    let event = Event::new("revoke")
        .add_attribute("sender", info.sender)
        .add_attribute("spender", spender)
        .add_attribute("token_id", token_id);
    let res = Response::new().add_event(event);

    Ok(res)
}

pub fn revoke_all(
    contract: Sg721Contract,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    operator: String,
) -> Result<Response, ContractError> {
    let operator_addr = deps.api.addr_validate(&operator)?;
    contract
        .operators
        .remove(deps.storage, (&info.sender, &operator_addr));

    let event = Event::new("revoke_all")
        .add_attribute("sender", info.sender)
        .add_attribute("operator", operator);
    let res = Response::new().add_event(event);

    Ok(res)
}

pub fn send_nft(
    contract: Sg721Contract,
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    receiving_contract: String,
    token_id: String,
    msg: Binary,
) -> Result<Response, ContractError> {
    // let hook = prepare_transfer_hook(
    //     deps.storage,
    //     info.sender.to_string(),
    //     &receiving_contract,
    //     &token_id,
    // )?;

    // Transfer token
    contract._transfer_nft(deps, &env, &info, &receiving_contract, &token_id)?;

    let send = Cw721ReceiveMsg {
        sender: info.sender.to_string(),
        token_id: token_id.clone(),
        msg,
    };

    // Send message
    let event = Event::new("send_nft")
        .add_attribute("sender", info.sender)
        .add_attribute("recipient", receiving_contract.to_string())
        .add_attribute("token_id", token_id);
    let res = Response::new()
        .add_message(send.into_cosmos_msg(receiving_contract)?)
        // .add_submessages(hook)
        .add_event(event);

    Ok(res)
}

pub fn transfer_nft(
    contract: Sg721Contract,
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    token_id: String,
) -> Result<Response, ContractError> {
    // let hook = prepare_transfer_hook(deps.storage, info.sender.to_string(), &recipient, &token_id)?;

    contract._transfer_nft(deps, &env, &info, &recipient, &token_id)?;

    let event = Event::new("transfer_nft")
        .add_attribute("sender", info.sender)
        .add_attribute("recipient", recipient)
        .add_attribute("token_id", token_id);
    let res = Response::new()
        // .add_submessages(hook)
        .add_event(event);

    Ok(res)
}

pub fn mint(
    contract: Sg721Contract,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: MintMsg<Extension>,
) -> Result<Response, ContractError> {
    let minter = contract.minter.load(deps.storage)?;

    if info.sender != minter {
        return Err(ContractError::Unauthorized {});
    }

    // create the token
    let token = TokenInfo {
        owner: deps.api.addr_validate(&msg.owner)?,
        approvals: vec![],
        token_uri: msg.token_uri,
        extension: msg.extension,
    };
    contract
        .tokens
        .update(deps.storage, &msg.token_id, |old| match old {
            Some(_) => Err(ContractError::Claimed {}),
            None => Ok(token),
        })?;

    contract.increment_tokens(deps.storage)?;

    Ok(Response::new()
        .add_attribute("action", "mint")
        .add_attribute("minter", info.sender)
        .add_attribute("owner", msg.owner)
        .add_attribute("token_id", msg.token_id))
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

// TODO: move to integration tests
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
