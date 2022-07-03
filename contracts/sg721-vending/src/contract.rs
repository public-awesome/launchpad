#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Decimal, Deps, DepsMut, Empty, Env, MessageInfo, StdResult};
use cw2::set_contract_version;

use sg1::checked_fair_burn;
use sg721::{CollectionInfo, InstantiateMsg, RoyaltyInfo, RoyaltyInfoResponse};
use sg_std::{Response, StargazeMsgWrapper};

use crate::ContractError;
use cw721::ContractInfoResponse;
use cw721_base::ContractError as BaseError;
use url::Url;

use crate::msg::{CollectionInfoResponse, ExecuteMsg, QueryMsg};
use crate::state::COLLECTION_INFO;

use factory::msg::{ParamsResponse, QueryMsg as FactoryQueryMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg-721";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MINTER_FACTORY: &str = "minter-factory-contract";

const CREATION_FEE: u128 = 5_000_000_000;
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

    // TODO:
    // query minter factory contract for params
    let res: ParamsResponse = deps
        .querier
        .query_wasm_smart(MINTER_FACTORY, &FactoryQueryMsg::Params {})?;
    println!("{:?}", res);

    // get allowed minter code ids
    // query sender's contract for code id and check for match

    let mut res = Response::new();
    checked_fair_burn(&info, CREATION_FEE, None, &mut res)?;

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

    Ok(res
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION)
        .add_attribute("image", image.to_string()))
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

#[cfg(test)]
mod tests {
    use super::*;

    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, Decimal};
    use sg_std::NATIVE_DENOM;

    #[test]
    fn proper_initialization_no_royalties() {
        let mut deps = mock_dependencies();
        let collection = String::from("collection0");

        let msg = InstantiateMsg {
            name: collection,
            symbol: String::from("BOBO"),
            minter: String::from("minter"),
            collection_info: CollectionInfo {
                creator: String::from("creator"),
                description: String::from("Stargaze Monkeys"),
                image: "https://example.com/image.png".to_string(),
                external_link: Some("https://example.com/external.html".to_string()),
                royalty_info: None,
            },
        };
        let info = mock_info("creator", &coins(CREATION_FEE, NATIVE_DENOM));

        // make sure instantiate has the burn messages
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(2, res.messages.len());

        // let's query the collection info
        let res = query(deps.as_ref(), mock_env(), QueryMsg::CollectionInfo {}).unwrap();
        let value: CollectionInfoResponse = from_binary(&res).unwrap();
        assert_eq!("https://example.com/image.png", value.image);
        assert_eq!("Stargaze Monkeys", value.description);
        assert_eq!(
            "https://example.com/external.html",
            value.external_link.unwrap()
        );
        assert_eq!(None, value.royalty_info);
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
            collection_info: CollectionInfo {
                creator: String::from("creator"),
                description: String::from("Stargaze Monkeys"),
                image: "https://example.com/image.png".to_string(),
                external_link: Some("https://example.com/external.html".to_string()),
                royalty_info: Some(RoyaltyInfoResponse {
                    payment_address: creator.clone(),
                    share: Decimal::percent(10),
                }),
            },
        };
        let info = mock_info("creator", &coins(CREATION_FEE, NATIVE_DENOM));

        // make sure instantiate has the burn messages
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(2, res.messages.len());

        // let's query the collection info
        let res = query(deps.as_ref(), mock_env(), QueryMsg::CollectionInfo {}).unwrap();
        let value: CollectionInfoResponse = from_binary(&res).unwrap();
        assert_eq!(
            Some(RoyaltyInfoResponse {
                payment_address: creator,
                share: Decimal::percent(10),
            }),
            value.royalty_info
        );
    }
}
