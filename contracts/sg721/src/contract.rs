#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::from_binary;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, StdResult};
use cw2::set_contract_version;

use cw721::TokensResponse;
use sg_std::burn_and_distribute_fee;
use sg_std::StargazeMsgWrapper;

use crate::state::FROZEN;
use crate::ContractError;
use cw721::ContractInfoResponse;
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

type Response = cosmwasm_std::Response<StargazeMsgWrapper>;
pub type Sg721Contract<'a> = cw721_base::Cw721Contract<'a, Empty, StargazeMsgWrapper>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let fee_msgs = burn_and_distribute_fee(&info, CREATION_FEE)?;

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
    FROZEN.save(deps.storage, &false)?;

    Ok(Response::default()
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION)
        .add_attribute("image", image.to_string())
        .add_messages(fee_msgs))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Freeze {} => execute_freeze(deps, info),
        ExecuteMsg::UpdateTokenURIs { base_token_uri } => {
            execute_update_token_uris(deps, env, info, base_token_uri)
        }
        _ => Sg721Contract::default()
            .execute(deps, env, info, msg.into())
            .map_err(ContractError::from),
    }
}

fn execute_freeze(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let sg721_contract = Sg721Contract::default();
    let minter = sg721_contract.minter.load(deps.storage)?;

    if info.sender != minter {
        return Err(ContractError::Unauthorized {});
    }

    FROZEN.save(deps.storage, &true)?;

    Ok(Response::new().add_attribute("action", "freeze"))
}
fn execute_update_token_uris(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    base_token_uri: String,
) -> Result<Response, ContractError> {
    let sg721_contract = Sg721Contract::default();
    let minter = sg721_contract.minter.load(deps.storage)?;
    let frozen = FROZEN.load(deps.storage)?;

    if info.sender != minter {
        return Err(ContractError::Unauthorized {});
    }

    if frozen {
        return Err(ContractError::Frozen {});
    }

    // iterate through all tokens
    let all_tokens_msg = QueryMsg::AllTokens {
        start_after: None,
        limit: None,
    };
    let tokens_response: TokensResponse =
        from_binary(&query(deps.as_ref(), env, all_tokens_msg).unwrap()).unwrap();
    for token_id in tokens_response.tokens {
        // update each token uri
        sg721_contract
            .tokens
            .update(deps.storage, &token_id, |token| match token {
                Some(mut token_info) => {
                    token_info.token_uri = Some(format!("{}/{}", base_token_uri, token_id));
                    token_info.extension = Empty {};
                    Ok(token_info)
                }
                None => Err(ContractError::TokenNotFound {
                    got: token_id.to_string(),
                }),
            })?;
    }

    Ok(Response::new()
        .add_attribute("action", "update_token_uri".to_string())
        .add_attribute("new_base_token_uri", base_token_uri.to_string()))
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
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, Decimal};
    use cw721::NftInfoResponse;
    use cw721_base::{ExecuteMsg as Cw721ExecuteMsg, MintMsg};
    use sg_std::NATIVE_DENOM;

    const CREATOR: &str = "creator";
    const MINTER: &str = "minter";

    #[test]
    fn proper_initialization_no_royalties() {
        let mut deps = mock_dependencies();
        let collection = String::from("collection0");

        let msg = InstantiateMsg {
            name: collection,
            symbol: String::from("BOBO"),
            minter: MINTER.to_string(),
            collection_info: CollectionInfo {
                creator: CREATOR.to_string(),
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

    #[test]
    fn update_token_uris() {
        // init contract
        let mut deps = mock_dependencies();
        let collection = String::from("collection_mint");

        let msg = InstantiateMsg {
            name: collection,
            symbol: String::from("BOBO"),
            minter: MINTER.to_string(),
            collection_info: CollectionInfo {
                creator: CREATOR.to_string(),
                description: String::from("Stargaze Monkeys"),
                image: "https://example.com/image.png".to_string(),
                external_link: None,
                royalty_info: None,
            },
        };
        let info = mock_info("creator", &coins(CREATION_FEE, NATIVE_DENOM));
        let _ = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // mint nft
        let token_id = "copypasta".to_string();
        let token_uri = "https://www.merriam-webster.com/dictionary/petrify".to_string();

        let exec_mint_msg = Cw721ExecuteMsg::Mint(MintMsg::<Empty> {
            token_id: token_id.clone(),
            owner: String::from("medusa"),
            token_uri: Some(token_uri.clone()),
            extension: Empty {},
        });

        let allowed = mock_info(MINTER, &[]);
        let _ = Sg721Contract::default()
            .execute(deps.as_mut(), mock_env(), allowed.clone(), exec_mint_msg)
            .unwrap();

        let query_msg: QueryMsg = QueryMsg::NftInfo {
            token_id: (&token_id).to_string(),
        };

        // confirm response is the same
        let res: NftInfoResponse<Empty> =
            from_binary(&query(deps.as_ref(), mock_env(), query_msg.clone()).unwrap()).unwrap();
        assert_eq!(res.token_uri, Some(token_uri));

        // update base token uri
        let new_base_token_uri: String = "ipfs://new_base_token_uri_hash".to_string();
        let exec_update_token_uris_msg = ExecuteMsg::UpdateTokenURIs {
            base_token_uri: new_base_token_uri.clone(),
        };
        let _ = execute(
            deps.as_mut(),
            mock_env(),
            allowed.clone(),
            exec_update_token_uris_msg,
        )
        .unwrap();

        let res: NftInfoResponse<Empty> =
            from_binary(&query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();

        assert_eq!(
            res.token_uri,
            Some(format!("{}/{}", new_base_token_uri, token_id))
        );

        // freeze contract
        let freeze_msg = ExecuteMsg::Freeze {};
        let _ = execute(deps.as_mut(), mock_env(), allowed.clone(), freeze_msg).unwrap();

        let even_newer_base_token_uri: String = "ipfs://even_newer_base_token_uri_hash".to_string();
        let exec_update_token_uris_msg = ExecuteMsg::UpdateTokenURIs {
            base_token_uri: even_newer_base_token_uri,
        };
        // error when trying to update token_uri after freeze
        let err = execute(
            deps.as_mut(),
            mock_env(),
            allowed,
            exec_update_token_uris_msg,
        )
        .unwrap_err();
        assert_eq!(err.to_string(), ContractError::Frozen {}.to_string());
    }
}
