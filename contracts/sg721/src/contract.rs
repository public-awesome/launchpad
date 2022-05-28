use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, RoyaltyInfoResponse, TransferHookMsg};
use crate::state::{CollectionInfo, RoyaltyInfo, COLLECTION_INFO, TRANSFER_HOOKS};
use crate::ContractError;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, ContractInfoResponse, Deps, DepsMut, Empty, Env, Event, MessageInfo,
    QueryRequest, StdResult, Storage, WasmMsg, WasmQuery,
};
use cw2::set_contract_version;
use cw721::{ContractInfoResponse as Cw721ContractInfoResponse, Cw721ReceiveMsg};
use cw721_base::state::TokenInfo;
use cw721_base::MintMsg;
use cw_utils::Expiration;
use sg1::checked_fair_burn;
use sg_std::{Response, StargazeMsgWrapper, SubMsg};
use url::Url;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg-721";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const CREATION_FEE: u128 = 1_000_000_000;
const MAX_DESCRIPTION_LENGTH: u32 = 512;
const REPLY_TRANSFER_HOOK: u64 = 1;

pub type BaseContract<'a> = cw721_base::Cw721Contract<'a, Empty, StargazeMsgWrapper>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let base = BaseContract::default();
    let mut res = Response::new();

    checked_fair_burn(&info, CREATION_FEE, None, &mut res)?;

    // cw721 instantiation
    let info = Cw721ContractInfoResponse {
        name: msg.name,
        symbol: msg.symbol,
    };
    base.contract_info.save(deps.storage, &info)?;

    let minter = deps.api.addr_validate(&msg.minter)?;

    // TODO: get this from the chain
    let code_ids = vec![1, 2, 3];

    // make sure collection can only be instantiated with registered minters
    let query = QueryRequest::Wasm(WasmQuery::ContractInfo {
        contract_addr: minter.to_string(),
    });
    let info_res: ContractInfoResponse = deps.querier.query(&query)?;
    let code_id = info_res.code_id;
    if code_ids.contains(&code_id) {
        return Err(ContractError::InvalidMinterCodeId { code_id });
    }

    base.minter.save(deps.storage, &minter)?;

    // sg721 instantiation
    if msg.collection_info.description.len() > MAX_DESCRIPTION_LENGTH as usize {
        return Err(ContractError::DescriptionTooLong {});
    }

    Url::parse(&msg.collection_info.image)?;

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

    if let Some(hook) = msg.transfer_hook {
        TRANSFER_HOOKS.add_hook(deps.storage, deps.api.addr_validate(&hook)?)?;
    }

    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute<T>(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg<T>,
) -> Result<Response, ContractError> {
    let base = BaseContract::default();
    match msg {
        ExecuteMsg::TransferNft {
            recipient,
            token_id,
        } => transfer_nft(base, deps, env, info, recipient, token_id),
        ExecuteMsg::SendNft {
            contract,
            token_id,
            msg,
        } => send_nft(base, deps, env, info, contract, token_id, msg),
        ExecuteMsg::Approve {
            spender,
            token_id,
            expires,
        } => approve(base, deps, env, info, spender, token_id, expires),
        ExecuteMsg::Revoke { spender, token_id } => {
            revoke(base, deps, env, info, spender, token_id)
        }
        ExecuteMsg::ApproveAll { operator, expires } => {
            approve_all(base, deps, env, info, operator, expires)
        }
        ExecuteMsg::RevokeAll { operator } => revoke_all(base, deps, env, info, operator),
        ExecuteMsg::Burn { token_id } => burn(base, deps, env, info, token_id),
        ExecuteMsg::Mint(msg) => mint(base, deps, env, info, msg),
    }
}

pub fn mint<T>(
    base: BaseContract,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: MintMsg<T>,
) -> Result<Response, ContractError> {
    let minter = base.minter.load(deps.storage)?;

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
    base.tokens
        .update(deps.storage, &msg.token_id, |old| match old {
            Some(_) => Err(ContractError::Claimed {}),
            None => Ok(token),
        })?;

    base.increment_tokens(deps.storage)?;

    Ok(Response::new()
        .add_attribute("action", "mint")
        .add_attribute("minter", info.sender)
        .add_attribute("owner", msg.owner)
        .add_attribute("token_id", msg.token_id))
}

fn transfer_nft(
    base: BaseContract,
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    token_id: String,
) -> Result<Response, ContractError> {
    let hook = prepare_transfer_hook(deps.storage, info.sender.to_string(), &recipient, &token_id)?;

    base._transfer_nft(deps, &env, &info, &recipient, &token_id)?;

    let event = Event::new("transfer_nft")
        .add_attribute("sender", info.sender)
        .add_attribute("recipient", recipient)
        .add_attribute("token_id", token_id);
    let res = Response::new().add_submessages(hook).add_event(event);

    Ok(res)
}

fn send_nft(
    base: BaseContract,
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    contract: String,
    token_id: String,
    msg: Binary,
) -> Result<Response, ContractError> {
    let hook = prepare_transfer_hook(deps.storage, info.sender.to_string(), &contract, &token_id)?;

    // Transfer token
    base._transfer_nft(deps, &env, &info, &contract, &token_id)?;

    let send = Cw721ReceiveMsg {
        sender: info.sender.to_string(),
        token_id: token_id.clone(),
        msg,
    };

    // Send message
    let event = Event::new("send_nft")
        .add_attribute("sender", info.sender)
        .add_attribute("recipient", contract.to_string())
        .add_attribute("token_id", token_id);
    let res = Response::new()
        .add_message(send.into_cosmos_msg(contract)?)
        .add_submessages(hook)
        .add_event(event);

    Ok(res)
}

fn approve(
    base: BaseContract,
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    spender: String,
    token_id: String,
    expires: Option<Expiration>,
) -> Result<Response, ContractError> {
    base._update_approvals(deps, &env, &info, &spender, &token_id, true, expires)?;

    let event = Event::new("approve")
        .add_attribute("sender", info.sender)
        .add_attribute("spender", spender)
        .add_attribute("token_id", token_id);
    let res = Response::new().add_event(event);

    Ok(res)
}

fn revoke(
    base: BaseContract,
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    spender: String,
    token_id: String,
) -> Result<Response, ContractError> {
    base._update_approvals(deps, &env, &info, &spender, &token_id, false, None)?;

    let event = Event::new("revoke")
        .add_attribute("sender", info.sender)
        .add_attribute("spender", spender)
        .add_attribute("token_id", token_id);
    let res = Response::new().add_event(event);

    Ok(res)
}

fn approve_all(
    base: BaseContract,
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
    base.operators
        .save(deps.storage, (&info.sender, &operator_addr), &expires)?;

    let event = Event::new("approve_all")
        .add_attribute("sender", info.sender)
        .add_attribute("operator", operator);
    let res = Response::new().add_event(event);

    Ok(res)
}

fn revoke_all(
    base: BaseContract,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    operator: String,
) -> Result<Response, ContractError> {
    let operator_addr = deps.api.addr_validate(&operator)?;
    base.operators
        .remove(deps.storage, (&info.sender, &operator_addr));

    let event = Event::new("revoke_all")
        .add_attribute("sender", info.sender)
        .add_attribute("operator", operator);
    let res = Response::new().add_event(event);

    Ok(res)
}

fn burn(
    base: BaseContract,
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {
    let token = base.tokens.load(deps.storage, &token_id)?;
    base.check_can_send(deps.as_ref(), &env, &info, &token)?;

    base.tokens.remove(deps.storage, &token_id)?;
    base.decrement_tokens(deps.storage)?;

    let event = Event::new("burn")
        .add_attribute("sender", info.sender)
        .add_attribute("token_id", token_id);
    let res = Response::new().add_event(event);

    Ok(res)
}

fn prepare_transfer_hook(
    store: &dyn Storage,
    sender: String,
    recipient: &str,
    token_id: &str,
) -> StdResult<Vec<SubMsg>> {
    let submsgs = TRANSFER_HOOKS.prepare_hooks(store, |h| {
        let msg = TransferHookMsg {
            sender: sender.to_string(),
            recipient: recipient.to_string(),
            token_id: token_id.to_string(),
        };
        let execute = WasmMsg::Execute {
            contract_addr: h.to_string(),
            msg: msg.into_binary()?,
            funds: vec![],
        };
        Ok(SubMsg::reply_on_error(execute, REPLY_TRANSFER_HOOK))
    })?;

    Ok(submsgs)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::CollectionInfo {} => to_binary(&query_config(deps)?),
        _ => BaseContract::default().query(deps, env, msg.into()),
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
            transfer_hook: None,
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
            transfer_hook: None,
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
