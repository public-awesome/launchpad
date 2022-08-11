#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cw721_base::state::TokenInfo;
use cw721_base::Extension;
use url::Url;

use cosmwasm_std::{Binary, Decimal, Deps, DepsMut, Env, Event, MessageInfo, StdResult};
use cw2::set_contract_version;
use cw721::{ContractInfoResponse, Cw721ReceiveMsg};
use cw_utils::{nonpayable, Expiration};

use sg721::{CollectionInfo, InstantiateMsg, MintMsg, RoyaltyInfo, RoyaltyInfoResponse};
use sg_std::Response;

use crate::msg::CollectionInfoResponse;
use crate::state::{COLLECTION_INFO, READY};
use crate::{ContractError, Cw721Base};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg721-base";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MAX_DESCRIPTION_LENGTH: u32 = 512;

pub fn _instantiate(
    contract: Cw721Base,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    create(contract, deps, info, msg)
}

pub fn create(
    contract: Cw721Base,
    deps: DepsMut,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // no funds should be sent to this contract
    nonpayable(&info)?;

    // cw721 instantiation
    let info = ContractInfoResponse {
        name: msg.name,
        symbol: msg.symbol,
    };
    contract.contract_info.save(deps.storage, &info)?;

    let minter = deps.api.addr_validate(&msg.minter)?;
    contract.minter.save(deps.storage, &minter)?;

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

    READY.save(deps.storage, &false)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION)
        .add_attribute("collection_name", info.name)
        .add_attribute("image", image.to_string()))
}

/// Called by the minter reply handler after instantiation. Now we can query
/// the factory and minter to verify that the collection creation is authorized.
pub fn ready(
    _contract: Cw721Base,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    println!("ready");

    let minter = Cw721Base::default().minter.load(deps.storage)?;
    if minter != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    READY.save(deps.storage, &true)?;

    Ok(Response::new())
}

pub fn approve(
    contract: Cw721Base,
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    spender: String,
    token_id: String,
    expires: Option<Expiration>,
) -> Result<Response, ContractError> {
    if !READY.load(deps.storage)? {
        return Err(ContractError::NotReady {});
    }

    contract._update_approvals(deps, &env, &info, &spender, &token_id, true, expires)?;

    let event = Event::new("approve")
        .add_attribute("sender", info.sender)
        .add_attribute("spender", spender)
        .add_attribute("token_id", token_id);
    let res = Response::new().add_event(event);

    Ok(res)
}

pub fn approve_all(
    contract: Cw721Base,
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    operator: String,
    expires: Option<Expiration>,
) -> Result<Response, ContractError> {
    if !READY.load(deps.storage)? {
        return Err(ContractError::NotReady {});
    }
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
    contract: Cw721Base,
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {
    if !READY.load(deps.storage)? {
        return Err(ContractError::NotReady {});
    }
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
    contract: Cw721Base,
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    spender: String,
    token_id: String,
) -> Result<Response, ContractError> {
    if !READY.load(deps.storage)? {
        return Err(ContractError::NotReady {});
    }
    contract._update_approvals(deps, &env, &info, &spender, &token_id, false, None)?;

    let event = Event::new("revoke")
        .add_attribute("sender", info.sender)
        .add_attribute("spender", spender)
        .add_attribute("token_id", token_id);
    let res = Response::new().add_event(event);

    Ok(res)
}

pub fn revoke_all(
    contract: Cw721Base,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    operator: String,
) -> Result<Response, ContractError> {
    if !READY.load(deps.storage)? {
        return Err(ContractError::NotReady {});
    }
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
    contract: Cw721Base,
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    receiving_contract: String,
    token_id: String,
    msg: Binary,
) -> Result<Response, ContractError> {
    if !READY.load(deps.storage)? {
        return Err(ContractError::NotReady {});
    }
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
        .add_event(event);

    Ok(res)
}

pub fn transfer_nft(
    contract: Cw721Base,
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    token_id: String,
) -> Result<Response, ContractError> {
    if !READY.load(deps.storage)? {
        return Err(ContractError::NotReady {});
    }
    contract._transfer_nft(deps, &env, &info, &recipient, &token_id)?;

    let event = Event::new("transfer_nft")
        .add_attribute("sender", info.sender)
        .add_attribute("recipient", recipient)
        .add_attribute("token_id", token_id);
    let res = Response::new().add_event(event);

    Ok(res)
}

pub fn mint(
    contract: Cw721Base,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: MintMsg<Extension>,
) -> Result<Response, ContractError> {
    if !READY.load(deps.storage)? {
        return Err(ContractError::NotReady {});
    }
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

pub fn query_collection_info(deps: Deps) -> StdResult<CollectionInfoResponse> {
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
