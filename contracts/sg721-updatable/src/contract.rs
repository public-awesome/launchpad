use semver::Version;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure, to_binary, Binary, Decimal, Deps, DepsMut, Empty, Env, Event, MessageInfo, StdError,
    StdResult, Uint128,
};
use cw2::set_contract_version;
use cw_utils::{nonpayable, Expiration, DAY};
use sg_std::fees::burn_and_distribute_fee;
use sg_std::StargazeMsgWrapper;

use crate::ContractError;
use cw721::ContractInfoResponse;
use url::Url;

use crate::msg::{
    CollectionInfoResponse, ExecuteMsg, InstantiateMsg, QueryMsg, RoyaltyInfoResponse,
};
use crate::state::{
    CollectionInfo, RoyaltyInfo, COLLECTION_INFO, FROZEN_TOKEN_METADATA, ROYALTY_UPDATED_AT,
};

// version info for migration info
const COMPATIBLE_MIGRATION_CONTRACT_NAME: &str = "crates.io:sg-721";
const CONTRACT_NAME: &str = "crates.io:sg721-updatable";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const CREATION_FEE: u128 = 1_000_000_000;
const MAX_ROYALTY_SHARE_BPS: u64 = 1_000;
const MAX_ROYALTY_SHARE_DELTA_BPS: u64 = 200;

// Disable instantiation for production build
#[cfg(target = "wasm32-unknown-unknown")]
const ENABLE_INSTANTIATE: bool = false;

#[cfg(not(target = "wasm32-unknown-unknown"))]
const ENABLE_INSTANTIATE: bool = true;

type Response = cosmwasm_std::Response<StargazeMsgWrapper>;
pub type Sg721Contract<'a> = cw721_base::Cw721Contract<'a, Empty, StargazeMsgWrapper>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    if !ENABLE_INSTANTIATE {
        return Err(ContractError::Unauthorized {});
    }
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let fee_msgs = burn_and_distribute_fee(env.clone(), &info, CREATION_FEE)?;

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
    if msg.collection_info.description.len() > 256 {
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

    FROZEN_TOKEN_METADATA.save(deps.storage, &false)?;
    ROYALTY_UPDATED_AT.save(deps.storage, &env.block.time)?;
    COLLECTION_INFO.save(deps.storage, &collection_info)?;

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
    msg: ExecuteMsg<Empty>,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::FreezeTokenMetadata {} => execute_freeze_token_metadata(deps, env, info),
        ExecuteMsg::UpdateTokenMetadata {
            token_id,
            token_uri,
        } => execute_update_token_metadata(deps, env, info, token_id, token_uri),
        ExecuteMsg::UpdateRoyaltyInfo {
            payment_address,
            share_bps,
        } => execute_update_royalty_info(deps, info, env, payment_address, share_bps),
        _ => Sg721Contract::default()
            .execute(deps, env, info, msg.into())
            .map_err(|e| e.into()),
    }
}

pub fn execute_update_royalty_info(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    payment_address: String,
    share_bps: u64,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    let mut collection_info = COLLECTION_INFO.load(deps.storage)?;
    // check sender is authorized
    if info.sender != collection_info.creator {
        return Err(ContractError::Unauthorized {});
    }

    let payment_addr = deps.api.addr_validate(&payment_address)?;
    let share = Decimal::percent(share_bps) / Uint128::from(100u128);
    let max_share_limit = Decimal::percent(MAX_ROYALTY_SHARE_BPS) / Uint128::from(100u128);
    let max_share_delta = Decimal::percent(MAX_ROYALTY_SHARE_DELTA_BPS) / Uint128::from(100u128);
    ensure!(
        share <= max_share_limit,
        // multiply by 100 to get the percentage for error message
        ContractError::InvalidRoyalties(format!(
            "Share percentage cannot be greater than {}%",
            max_share_limit * Uint128::from(100u128)
        ))
    );

    let updated_at = ROYALTY_UPDATED_AT.load(deps.storage)?;

    if let Some(old_royalty_info) = collection_info.royalty_info {
        // make sure the update time is at least 24 hours after the last one
        let end = (Expiration::AtTime(updated_at) + DAY)?;
        ensure!(
            end.is_expired(&env.block),
            ContractError::RoyaltyUpdateTooSoon {}
        );
        // Make sure the share is not increased more than MAX_ROYALTY_SHARE_DELTA_BPS at a time
        if share > old_royalty_info.share {
            let share_delta = share - old_royalty_info.share;
            ensure!(
                share_delta <= max_share_delta,
                ContractError::InvalidRoyalties(format!(
                    "Share increase cannot be greater than {}%",
                    max_share_delta * Uint128::from(100u128)
                ))
            );
        }
    } else {
        ensure!(
            share <= max_share_delta,
            ContractError::InvalidRoyalties(format!(
                "Share increase cannot be greater than {}%",
                max_share_delta * Uint128::from(100u128)
            ))
        );
    }

    collection_info.royalty_info = Some(RoyaltyInfo {
        payment_address: payment_addr,
        share,
    });

    COLLECTION_INFO.save(deps.storage, &collection_info)?;
    ROYALTY_UPDATED_AT.save(deps.storage, &env.block.time)?;

    let event = Event::new("update_royalty_info")
        .add_attribute("sender", info.sender)
        .add_attribute("payment_address", payment_address)
        .add_attribute("share", share.to_string())
        .add_attribute("updated_at", env.block.time.to_string());
    Ok(Response::new().add_event(event))
}

pub fn execute_freeze_token_metadata(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    // Check if sender is creator
    let collection_info: CollectionInfoResponse = query_config(deps.as_ref())?;
    if info.sender != collection_info.creator {
        return Err(ContractError::Unauthorized {});
    }

    FROZEN_TOKEN_METADATA.save(deps.storage, &true)?;

    let event = Event::new("freeze_token_metadata").add_attribute("frozen", "true");
    Ok(Response::new().add_event(event))
}

pub fn execute_update_token_metadata(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_id: String,
    token_uri: Option<String>,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    // Check if sender is creator
    let creator = deps.api.addr_validate(info.sender.as_ref())?;
    let collection_info: CollectionInfoResponse = query_config(deps.as_ref())?;
    if creator != collection_info.creator {
        return Err(ContractError::Unauthorized {});
    }

    // Check if token metadata is frozen
    let frozen = FROZEN_TOKEN_METADATA.load(deps.storage)?;
    if frozen {
        return Err(ContractError::TokenMetadataFrozen {});
    }

    // Update token metadata
    Sg721Contract::default()
        .tokens
        .update(deps.storage, &token_id, |token| match token {
            Some(mut token_info) => {
                token_info.token_uri = token_uri.clone();
                Ok(token_info)
            }
            None => Err(ContractError::TokenIdNotFound {}),
        })?;

    let event = Event::new("update_update_token_metadata")
        .add_attribute("sender", info.sender)
        .add_attribute("token_id", token_id)
        .add_attribute("token_uri", token_uri.unwrap_or_default());
    Ok(Response::new().add_event(event))
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: Empty) -> Result<Response, ContractError> {
    let current_version = cw2::get_contract_version(deps.storage)?;
    if ![CONTRACT_NAME, COMPATIBLE_MIGRATION_CONTRACT_NAME]
        .contains(&current_version.contract.as_str())
    {
        return Err(StdError::generic_err("Cannot upgrade to a different contract").into());
    }
    let version: Version = current_version
        .version
        .parse()
        .map_err(|_| StdError::generic_err("Invalid contract version"))?;
    let new_version: Version = CONTRACT_VERSION
        .parse()
        .map_err(|_| StdError::generic_err("Invalid contract version"))?;

    if version > new_version {
        return Err(StdError::generic_err("Cannot upgrade to a previous contract version").into());
    }
    // if same version return
    if version == new_version {
        return Ok(Response::new());
    }

    FROZEN_TOKEN_METADATA.save(deps.storage, &false)?;
    // setting updated_at to 1 day ago to make updating royalties possible right after a migration
    ROYALTY_UPDATED_AT.save(deps.storage, &_env.block.time.minus_seconds(86400))?;
    // set new contract version
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::state::CollectionInfo;
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
