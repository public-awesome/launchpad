use cosmwasm_std::{Addr, Binary, Decimal, Timestamp};
use cw_utils::Expiration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MintMsg<T> {
    /// Unique ID of the NFT
    pub token_id: String,
    /// The owner of the newly minter NFT
    pub owner: String,
    /// Universal resource identifier for this NFT
    /// Should point to a JSON file that conforms to the ERC721
    /// Metadata JSON Schema
    pub token_uri: Option<String>,
    /// Any custom extension used by this contract
    pub extension: T,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg<T> {
    /// Called by the minter to put a collection contract in the ready state.
    /// When ready it means the factory and minter are properly setup.
    _Ready {},

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
    /// Update specific collection info fields
    UpdateCollectionInfo {
        collection_info: UpdateCollectionInfoMsg<RoyaltyInfoResponse>,
    },
    /// Called by the minter to update trading start time
    UpdateTradingStartTime(Option<Timestamp>),
    // Freeze collection info from further updates
    FreezeCollectionInfo,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollectionInfo<T> {
    pub creator: String,
    pub description: String,
    pub image: String,
    pub external_link: Option<String>,
    pub trading_start_time: Option<Timestamp>,
    pub royalty_info: Option<T>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateCollectionInfoMsg<T> {
    pub description: Option<String>,
    pub image: Option<String>,
    pub external_link: Option<Option<String>>,
    pub royalty_info: Option<Option<T>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct RoyaltyInfoResponse {
    pub payment_address: String,
    pub share: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub name: String,
    pub symbol: String,
    pub minter: String,
    pub collection_info: CollectionInfo<RoyaltyInfoResponse>,
}
