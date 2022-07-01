#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, QueryRequest, StdResult, WasmQuery,
};
use cw2::set_contract_version;

use sg1::checked_fair_burn;
use sg_std::{Response, StargazeMsgWrapper};

use crate::ContractError;
use cw721::ContractInfoResponse;
use cw721_base::ContractError as BaseError;
use url::Url;

use crate::msg::{
    CollectionInfoResponse, ExecuteMsg, InstantiateMsg, QueryMsg, RoyaltyInfoResponse,
};
use crate::state::{CollectionInfo, RoyaltyInfo, COLLECTION_INFO};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg-721";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const CREATION_FEE: u128 = 1_000_000_000;
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

    // check if minter is a contract and save
    // TODO: check length of contract address
    println!("minter: {}", msg.minter);
    println!("minter len: {}", msg.minter.len());

    if msg.minter.len() != 64 {
        return Err(ContractError::InvalidMinter {});
    }

    let minter = deps.api.addr_validate(&msg.minter)?;
    // let query = QueryRequest::Wasm(WasmQuery::ContractInfo {
    //     contract_addr: minter.to_string(),
    // });
    // deps.querier.query(&query)?;
    Sg721Contract::default()
        .minter
        .save(deps.storage, &minter)?;

    println!("here");

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
            share: royalty_info.share_validate()?,
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::state::CollectionInfo;
    use cosmwasm_std::coins;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use sg_std::NATIVE_DENOM;

    #[test]
    fn check_minter_is_a_contract() {
        let mut deps = mock_dependencies();
        let collection = String::from("collection0");

        let msg = InstantiateMsg {
            name: collection,
            symbol: String::from("BOBO"),
            // minter: String::from("minter"),
            minter: String::from(
                "stars1fhjm2yshl8qef3dvlj03sdckqjq5k84nlwcrqeqyh0c4gvjxv63qef9ccd",
            ),
            // minter: String::from("stars1dnt5k4wfyx27sh9vnkmysersyqesr0d5rkn6tz"),
            collection_info: CollectionInfo {
                creator: String::from("creator"),
                description: String::from("Stargaze Monkeys"),
                image: "https://example.com/image.png".to_string(),
                external_link: Some("https://example.com/external.html".to_string()),
                royalty_info: None,
            },
        };
        let info = mock_info("creator", &coins(CREATION_FEE, NATIVE_DENOM));

        // fails because minter is not a contract
        let err = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        println!("{:?}", err);
    }
}
