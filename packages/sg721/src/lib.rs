use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    coin, Addr, BankMsg, Binary, Decimal, Event, StdError, StdResult, Timestamp, Uint128,
};
use cw721_base::MintMsg;
use cw_utils::Expiration;
use sg_std::{Response, SubMsg};

#[cw_serde]
pub enum ExecuteMsg<T, E> {
    /// Transfer is a base message to move a token to another account without triggering actions
    TransferNft {
        recipient: String,
        token_id: String,
    },
    /// Send is a base message to transfer a token to a contract and trigger an action
    /// on the receiving contract.
    SendNft {
        contract: String,
        token_id: String,
        msg: Binary,
    },
    /// Allows operator to transfer / send the token from the owner's account.
    /// If expiration is set, then this allowance has a time/height limit
    Approve {
        spender: String,
        token_id: String,
        expires: Option<Expiration>,
    },
    /// Remove previously granted Approval
    Revoke {
        spender: String,
        token_id: String,
    },
    /// Allows operator to transfer / send any token from the owner's account.
    /// If expiration is set, then this allowance has a time/height limit
    ApproveAll {
        operator: String,
        expires: Option<Expiration>,
    },
    /// Remove previously granted ApproveAll permission
    RevokeAll {
        operator: String,
    },
    /// Mint a new NFT, can only be called by the contract minter
    Mint(MintMsg<T>),
    /// Burn an NFT the sender has access to
    Burn {
        token_id: String,
    },
    /// Extension msg
    Extension {
        msg: E,
    },
    /// Update specific collection info fields
    UpdateCollectionInfo {
        collection_info: UpdateCollectionInfoMsg<RoyaltyInfoResponse>,
    },
    /// Called by the minter to update trading start time
    UpdateStartTradingTime(Option<Timestamp>),
    // Freeze collection info from further updates
    FreezeCollectionInfo,
}

#[cw_serde]
pub struct CollectionInfo<T> {
    pub creator: String,
    pub description: String,
    pub image: String,
    pub external_link: Option<String>,
    pub explicit_content: Option<bool>,
    pub start_trading_time: Option<Timestamp>,
    pub royalty_info: Option<T>,
}

#[cw_serde]
pub struct UpdateCollectionInfoMsg<T> {
    pub description: Option<String>,
    pub image: Option<String>,
    pub external_link: Option<Option<String>>,
    pub explicit_content: Option<bool>,
    pub royalty_info: Option<Option<T>>,
}

#[cw_serde]
pub struct RoyaltyInfo {
    pub payment_address: Addr,
    pub share: Decimal,
}

// allows easy conversion from RoyaltyInfo to RoyaltyInfoResponse
impl RoyaltyInfo {
    pub fn to_response(&self) -> RoyaltyInfoResponse {
        RoyaltyInfoResponse {
            payment_address: self.payment_address.to_string(),
            share: self.share,
        }
    }
}

#[cw_serde]
pub struct RoyaltyInfoResponse {
    pub payment_address: String,
    pub share: Decimal,
}

impl RoyaltyInfoResponse {
    pub fn payout(
        &self,
        collection: Addr,
        payment: Uint128,
        protocol_fee: Uint128,
        finders_fee: Uint128,
        res: &mut Response,
    ) -> StdResult<()> {
        // let amount = coin((payment * self.share).u128(), NATIVE_DENOM);
        let amount = coin((payment * self.share).u128(), "ustars");
        if payment < (protocol_fee + finders_fee + amount.amount) {
            return Err(StdError::generic_err("Fees exceed payment"));
        }
        res.messages.push(SubMsg::new(BankMsg::Send {
            to_address: self.payment_address.to_string(),
            amount: vec![amount.clone()],
        }));

        let event = Event::new("royalty-payout")
            .add_attribute("collection", collection.to_string())
            .add_attribute("amount", amount.to_string())
            .add_attribute("recipient", self.payment_address.to_string());
        res.events.push(event);

        Ok(())
    }
}

#[cw_serde]
pub struct InstantiateMsg {
    pub name: String,
    pub symbol: String,
    pub minter: String,
    pub collection_info: CollectionInfo<RoyaltyInfoResponse>,
}
