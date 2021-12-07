use crate::error::ContractError;
use crate::msg::{
    BidResponse, BidsResponse, CurrentAskResponse, ExecuteMsg, InstantiateMsg, QueryMsg,
};
use crate::state::{Ask, Bid, TOKEN_ASKS, TOKEN_BIDS};
use cosmwasm_std::{
    entry_point, has_coins, to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Order, Response, StdResult, WasmMsg,
};
use cw2::set_contract_version;
use cw721::{Cw721ExecuteMsg, OwnerOfResponse};
use cw_storage_plus::Bound;

// Version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg-marketplace";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Query limits
const DEFAULT_QUERY_LIMIT: u32 = 10;
const MAX_QUERY_LIMIT: u32 = 30;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetBid {
            collection,
            token_id,
            bid,
        } => execute_set_bid(deps, env, info, collection, token_id, bid),
        ExecuteMsg::RemoveBid {
            collection,
            token_id,
            bidder,
        } => execute_remove_bid(deps, env, info, collection, token_id, bidder),
        ExecuteMsg::SetAsk {
            collection,
            token_id,
            ask,
        } => execute_set_ask(deps, env, info, collection, token_id, ask),
        ExecuteMsg::RemoveAsk {
            collection,
            token_id,
        } => execute_remove_ask(deps, env, info, collection, token_id),
        ExecuteMsg::AcceptBid {
            collection,
            token_id,
            bid,
        } => execute_accept_bid(deps, env, info, collection, token_id, bid),
    }
}

/// Anyone may place a bid on a minted token. By placing a bid, the bidder sends a native Coin to the market contract.
pub fn execute_set_bid(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection: Addr,
    token_id: String,
    bid: Bid,
) -> Result<Response, ContractError> {
    if info.sender != bid.bidder {
        return Err(ContractError::Unauthorized {});
    }

    // Check bid is valid
    bid.is_valid()?;

    // Check sent amount matches bid
    if !has_coins(&info.funds, &bid.amount) {
        return Err(ContractError::InsufficientBidFunds {});
    }

    let mut res = Response::new();

    // Check bidder has existing bid, if so remove existing bid
    if let Some(existing_bid) = query_bid(deps.as_ref(), &collection, &token_id, &bid.bidder)?.bid {
        // Remove bid
        TOKEN_BIDS.remove(deps.storage, (&collection, &token_id, &bid.bidder));

        // Refund bidder msg
        let exec_refund_bidder = BankMsg::Send {
            to_address: existing_bid.bidder.to_string(),
            amount: vec![existing_bid.amount],
        };

        // Add refund bidder msg to response
        res = res.add_message(exec_refund_bidder)
    }

    // Check if bid meets ask criteria. has_coins() checks both amount and denom.
    // Finalize sale if so.
    let ask = query_current_ask(deps.as_ref(), &collection, &token_id)?.ask;
    if ask != None && has_coins(&[bid.amount.clone()], &ask.unwrap().amount) {
        // Remove ask
        TOKEN_ASKS.remove(deps.storage, (&collection, &token_id));

        // Include messages needed to finalize nft transfer and payout
        let msgs: Vec<CosmosMsg> = finalize_sale(
            deps,
            env,
            info,
            collection.clone(),
            token_id.clone(),
            bid.bidder.clone(),
            bid.amount.clone(),
        )?;

        res = res
            .add_attribute("method", "sale_finalized")
            .add_messages(msgs);
    } else {
        // Save bid
        TOKEN_BIDS.save(deps.storage, (&collection, &token_id, &bid.bidder), &bid)?;

        res = res.add_attribute("method", "set_bid");
    }

    Ok(res
        .add_attribute("collection", collection)
        .add_attribute("token_id", token_id)
        .add_attribute("bidder", bid.bidder)
        .add_attribute("amount", bid.amount.to_string()))
}

/// Removes a bid made by the bidder. Bidders can only remove their own bids
pub fn execute_remove_bid(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection: Addr,
    token_id: String,
    bidder: Addr,
) -> Result<Response, ContractError> {
    // Check bid exists for bidder
    let bid = query_bid(deps.as_ref(), &collection, &token_id, &bidder)?
        .bid
        .ok_or(ContractError::BidNotFound {})?;

    // Check sender is the bidder
    if info.sender != bid.bidder {
        return Err(ContractError::Unauthorized {});
    }

    // Remove bid
    TOKEN_BIDS.remove(deps.storage, (&collection, &token_id, &bidder));

    // Refund bidder
    let exec_refund_bidder = BankMsg::Send {
        to_address: bidder.to_string(),
        amount: vec![bid.amount],
    };

    Ok(Response::new()
        .add_attribute("method", "remove_bid")
        .add_attribute("collection", collection)
        .add_attribute("token_id", token_id)
        .add_attribute("bidder", bidder)
        .add_message(exec_refund_bidder))
}

/// An owner may set an Ask on their media. A bid is automatically fulfilled if it meets the asking price.
pub fn execute_set_ask(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection: Addr,
    token_id: String,
    ask: Ask,
) -> Result<Response, ContractError> {
    // Only the media onwer can call this
    let owner_of_response = check_only_owner(deps.as_ref(), &info, &collection, &token_id)?;
    // Check that approval has been set for marketplace contract
    if owner_of_response
        .approvals
        .iter()
        .map(|x| x.spender == env.contract.address)
        .len()
        != 1
    {
        return Err(ContractError::NeedsApproval {});
    }
    TOKEN_ASKS.save(deps.storage, (&collection, &token_id), &ask)?;
    Ok(Response::new()
        .add_attribute("method", "set_ask")
        .add_attribute("collection", collection)
        .add_attribute("token_id", token_id)
        .add_attribute("amount", ask.amount.to_string()))
}

/// Removes the ask on a particular media
pub fn execute_remove_ask(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection: Addr,
    token_id: String,
) -> Result<Response, ContractError> {
    // Only the media onwer can call this
    check_only_owner(deps.as_ref(), &info, &collection, &token_id)?;
    TOKEN_ASKS.remove(deps.storage, (&collection, &token_id));
    Ok(Response::new()
        .add_attribute("method", "remove_ask")
        .add_attribute("collection", collection)
        .add_attribute("token_id", token_id))
}

/// Owner can accept a bid which transfers funds as well as the media
pub fn execute_accept_bid(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection: Addr,
    token_id: String,
    bid: Bid,
) -> Result<Response, ContractError> {
    // Only the media onwer can call this
    check_only_owner(deps.as_ref(), &info, &collection, &token_id)?;

    // Remove ask
    TOKEN_ASKS.remove(deps.storage, (&collection, &token_id));

    // Remove accepted bid
    TOKEN_BIDS.remove(deps.storage, (&collection, &token_id, &bid.bidder));

    // Transfer funds and NFT
    let msgs = finalize_sale(
        deps,
        env,
        info,
        collection.clone(),
        token_id.clone(),
        bid.bidder.clone(),
        bid.amount.clone(),
    )?;

    Ok(Response::new()
        .add_attribute("method", "accept_bid")
        .add_attribute("collection", collection)
        .add_attribute("token_id", token_id)
        .add_attribute("bidder", bid.bidder)
        .add_messages(msgs))
}

/// Checks to enfore only nft owner can call
pub fn check_only_owner(
    deps: Deps,
    info: &MessageInfo,
    collection: &Addr,
    token_id: &str,
) -> Result<OwnerOfResponse, ContractError> {
    let owner: cw721::OwnerOfResponse = deps.querier.query_wasm_smart(
        collection,
        &cw721::Cw721QueryMsg::OwnerOf {
            token_id: token_id.to_string(),
            include_expired: None,
        },
    )?;
    if owner.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    Ok(owner)
}

/// Transfers funds and NFT, updates bid
pub fn finalize_sale(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    collection: Addr,
    token_id: String,
    recipient: Addr,
    amount: Coin,
) -> StdResult<Vec<CosmosMsg>> {
    // Payout bid
    let mut msgs: Vec<CosmosMsg> = payout(deps.as_ref(), &collection, &token_id, &amount)?;

    // Create transfer cw721 msg
    let cw721_transfer_msg = Cw721ExecuteMsg::TransferNft {
        token_id,
        recipient: recipient.to_string(),
    };
    let exec_cw721_transfer = WasmMsg::Execute {
        contract_addr: collection.to_string(),
        msg: to_binary(&cw721_transfer_msg)?,
        funds: vec![],
    };

    msgs.append(&mut vec![exec_cw721_transfer.into()]);

    Ok(msgs)
}

/// Payout a bid
pub fn payout(
    deps: Deps,
    collection: &Addr,
    token_id: &str,
    amount: &Coin,
) -> StdResult<Vec<CosmosMsg>> {
    // Will hold payment msgs
    let mut msgs: Vec<CosmosMsg> = vec![];

    // Get current token owner
    let owner: cw721::OwnerOfResponse = deps.querier.query_wasm_smart(
        collection,
        &cw721::Cw721QueryMsg::OwnerOf {
            token_id: token_id.to_string(),
            include_expired: None,
        },
    )?;

    let owner_share_msg = BankMsg::Send {
        to_address: owner.owner,
        amount: vec![Coin {
            amount: amount.amount,
            denom: amount.denom.clone(),
        }],
    };
    msgs.append(&mut vec![owner_share_msg.into()]);

    Ok(msgs)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::CurrentAsk {
            collection,
            token_id,
        } => to_binary(&query_current_ask(deps, &collection, &token_id)?),
        QueryMsg::Bid {
            collection,
            token_id,
            bidder,
        } => to_binary(&query_bid(deps, &collection, &token_id, &bidder)?),
        QueryMsg::Bids {
            collection,
            token_id,
            start_after,
            limit,
        } => to_binary(&query_bids(
            deps,
            &collection,
            &token_id,
            start_after,
            limit,
        )?),
    }
}

pub fn query_current_ask(
    deps: Deps,
    collection: &Addr,
    token_id: &str,
) -> StdResult<CurrentAskResponse> {
    let ask = TOKEN_ASKS.may_load(deps.storage, (collection, token_id))?;

    Ok(CurrentAskResponse { ask })
}

pub fn query_bid(
    deps: Deps,
    collection: &Addr,
    token_id: &str,
    bidder: &Addr,
) -> StdResult<BidResponse> {
    let bid = TOKEN_BIDS.may_load(deps.storage, (collection, token_id, bidder))?;

    Ok(BidResponse { bid })
}

pub fn query_bids(
    deps: Deps,
    collection: &Addr,
    token_id: &str,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<BidsResponse> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    let bids: StdResult<Vec<Bid>> = TOKEN_BIDS
        .prefix((collection, token_id))
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (_k, v) = item?;
            Ok(Bid {
                amount: v.amount,
                bidder: v.bidder,
                recipient: v.recipient,
            })
        })
        .collect();

    Ok(BidsResponse { bids: bids? })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coin, coins, from_binary};

    const CREATOR: &str = "creator";
    const COLLECTION: &str = "collection";
    const NATIVE_TOKEN_DENOM: &str = "ustars";
    const TOKEN_ID: &str = "123";

    fn setup_contract(deps: DepsMut) {
        let msg = InstantiateMsg {};
        let info = mock_info(CREATOR, &[]);
        let res = instantiate(deps, mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, NATIVE_TOKEN_DENOM));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn set_and_remove_bid() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());

        let broke = mock_info("broke", &[]);
        let bidder = mock_info("bidder", &coins(1000, NATIVE_TOKEN_DENOM));
        let recipient = mock_info("recipient", &coins(1000, NATIVE_TOKEN_DENOM));
        let random_addr = mock_info("random", &coins(1000, NATIVE_TOKEN_DENOM));

        // Ensure funds bidder has funds
        let bid = Bid {
            amount: coin(100, NATIVE_TOKEN_DENOM),
            bidder: broke.sender.clone(),
            recipient: recipient.sender.clone(),
        };
        let set_bid_msg = ExecuteMsg::SetBid {
            collection: Addr::unchecked(COLLECTION),
            token_id: TOKEN_ID.to_string(),
            bid,
        };

        // Bidder calls Set Bid successfully
        let res = execute(deps.as_mut(), mock_env(), broke, set_bid_msg);
        assert_eq!(res, Err(ContractError::InsufficientBidFunds {}));

        // Set bid
        let bid = Bid {
            amount: coin(100, NATIVE_TOKEN_DENOM),
            bidder: bidder.sender.clone(),
            recipient: recipient.sender,
        };
        let set_bid_msg = ExecuteMsg::SetBid {
            collection: Addr::unchecked(COLLECTION),
            token_id: TOKEN_ID.to_string(),
            bid: bid.clone(),
        };

        // Bidder calls Set Bid successfully
        let res = execute(deps.as_mut(), mock_env(), bidder.clone(), set_bid_msg);
        assert!(res.is_ok());

        // Query for bid
        let query_bid_msg = QueryMsg::Bid {
            collection: Addr::unchecked(COLLECTION),
            token_id: TOKEN_ID.to_string(),
            bidder: bidder.sender.clone(),
        };

        let q = query(deps.as_ref(), mock_env(), query_bid_msg).unwrap();
        let value: BidResponse = from_binary(&q).unwrap();
        assert_eq!(value, BidResponse { bid: Some(bid) });

        // Query for list of bids
        let bids_query_msg = QueryMsg::Bids {
            collection: Addr::unchecked(COLLECTION),
            token_id: TOKEN_ID.to_string(),
            start_after: None,
            limit: None,
        };
        let q = query(deps.as_ref(), mock_env(), bids_query_msg).unwrap();
        let value: BidsResponse = from_binary(&q).unwrap();
        assert_eq!(value.bids.len(), 1);

        // Remove bid
        let remove_bid_msg = ExecuteMsg::RemoveBid {
            collection: Addr::unchecked(COLLECTION),
            token_id: TOKEN_ID.to_string(),
            bidder: bidder.sender.clone(),
        };

        // Random address can't remove bid
        let res = execute(
            deps.as_mut(),
            mock_env(),
            random_addr,
            remove_bid_msg.clone(),
        );
        assert!(res.is_err());

        // Bidder can remove bid
        let res = execute(deps.as_mut(), mock_env(), bidder, remove_bid_msg).unwrap();

        // Check Bank msg was added for refund
        assert_eq!(1, res.messages.len());
    }

    #[test]
    fn set_ask() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());

        let ask = Ask {
            amount: coin(100, NATIVE_TOKEN_DENOM),
        };
        let set_ask = ExecuteMsg::SetAsk {
            collection: Addr::unchecked(COLLECTION),
            ask,
            token_id: TOKEN_ID.to_string(),
        };

        // Reject if not called by the media owner
        let not_allowed = mock_info("random", &[]);
        let err = execute(deps.as_mut(), mock_env(), not_allowed, set_ask);
        assert!(err.is_err());
    }
}
