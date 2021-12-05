use crate::error::ContractError;
use crate::state::Bid;
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, StdResult, Uint128,
    WasmMsg,
};
use cw721::{Cw721ExecuteMsg, OwnerOfResponse};

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

/// Checks if a bid is valid
pub fn is_valid_bid(bid: &Bid) -> Result<bool, ContractError> {
    let bid_amount = &bid.amount;

    // Check amount is not less than 100, or split share math will fail
    if bid_amount.amount < Uint128::new(100) {
        return Err(ContractError::InvalidBidTooLow {});
    }

    // Check amount is not zero
    if bid_amount.amount.is_zero() {
        return Err(ContractError::InvalidBidTooLow {});
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{coin, Addr};
    const NATIVE_TOKEN_DENOM: &str = "ustars";

    #[test]
    fn valid_bids() {
        let bidder: Addr = Addr::unchecked("bidder");
        let creator: Addr = Addr::unchecked("creator");

        // Normal bid
        let bid = Bid {
            amount: coin(100, NATIVE_TOKEN_DENOM),
            bidder: bidder.clone(),
            recipient: creator.clone(),
        };
        assert!(is_valid_bid(&bid).is_ok());

        // High number
        let bid = Bid {
            amount: coin(1000000000000, NATIVE_TOKEN_DENOM),
            bidder,
            recipient: creator,
        };
        assert!(is_valid_bid(&bid).is_ok());
    }

    #[test]
    fn invalid_bids() {
        let bidder: Addr = Addr::unchecked("bidder");
        let creator: Addr = Addr::unchecked("creator");

        // Low number
        let bid = Bid {
            amount: coin(1, NATIVE_TOKEN_DENOM),
            bidder,
            recipient: creator,
        };
        assert!(is_valid_bid(&bid).is_err());
    }
}
