use crate::{state::CollectionInfo, ContractError};
use cosmwasm_std::{to_binary, Binary, Decimal, StdResult};
use cw721_base::{msg::QueryMsg as Cw721QueryMsg, MintMsg};
use cw_utils::Expiration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub name: String,
    pub symbol: String,
    pub minter: String,
    pub collection_info: CollectionInfo<RoyaltyInfoResponse>,
    /// Address of contract to get a callback for every transfer
    pub transfer_hook: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RoyaltyInfoResponse {
    pub payment_address: String,
    pub share: Decimal,
}

impl RoyaltyInfoResponse {
    pub fn share_validate(&self) -> Result<Decimal, ContractError> {
        if self.share > Decimal::one() {
            return Err(ContractError::InvalidRoyalities {});
        }

        Ok(self.share)
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg<T> {
    /// Transfer is a base message to move a token to another account without triggering actions
    TransferNft { recipient: String, token_id: String },
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
    Revoke { spender: String, token_id: String },
    /// Allows operator to transfer / send any token from the owner's account.
    /// If expiration is set, then this allowance has a time/height limit
    ApproveAll {
        operator: String,
        expires: Option<Expiration>,
    },
    /// Remove previously granted ApproveAll permission
    RevokeAll { operator: String },
    /// Mint a new NFT, can only be called by the contract minter
    Mint(MintMsg<T>),
    /// Burn an NFT the sender has access to
    Burn { token_id: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    OwnerOf {
        token_id: String,
        include_expired: Option<bool>,
    },
    Approval {
        token_id: String,
        spender: String,
        include_expired: Option<bool>,
    },
    Approvals {
        token_id: String,
        include_expired: Option<bool>,
    },
    AllOperators {
        owner: String,
        include_expired: Option<bool>,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    NumTokens {},
    ContractInfo {},
    NftInfo {
        token_id: String,
    },
    AllNftInfo {
        token_id: String,
        include_expired: Option<bool>,
    },
    Tokens {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    AllTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    Minter {},
    CollectionInfo {},
}

impl From<QueryMsg> for Cw721QueryMsg {
    fn from(msg: QueryMsg) -> Cw721QueryMsg {
        match msg {
            QueryMsg::OwnerOf {
                token_id,
                include_expired,
            } => Cw721QueryMsg::OwnerOf {
                token_id,
                include_expired,
            },
            QueryMsg::Approval {
                token_id,
                spender,
                include_expired,
            } => Cw721QueryMsg::Approval {
                token_id,
                spender,
                include_expired,
            },
            QueryMsg::Approvals {
                token_id,
                include_expired,
            } => Cw721QueryMsg::Approvals {
                token_id,
                include_expired,
            },
            QueryMsg::AllOperators {
                owner,
                include_expired,
                start_after,
                limit,
            } => Cw721QueryMsg::AllOperators {
                owner,
                include_expired,
                start_after,
                limit,
            },
            QueryMsg::NumTokens {} => Cw721QueryMsg::NumTokens {},
            QueryMsg::ContractInfo {} => Cw721QueryMsg::ContractInfo {},
            QueryMsg::NftInfo { token_id } => Cw721QueryMsg::NftInfo { token_id },
            QueryMsg::AllNftInfo {
                token_id,
                include_expired,
            } => Cw721QueryMsg::AllNftInfo {
                token_id,
                include_expired,
            },
            QueryMsg::Tokens {
                owner,
                start_after,
                limit,
            } => Cw721QueryMsg::Tokens {
                owner,
                start_after,
                limit,
            },
            QueryMsg::AllTokens { start_after, limit } => {
                Cw721QueryMsg::AllTokens { start_after, limit }
            }
            QueryMsg::Minter {} => Cw721QueryMsg::Minter {},
            _ => unreachable!("cannot convert {:?} to Cw721QueryMsg", msg),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollectionInfoResponse {
    pub creator: String,
    pub description: String,
    pub image: String,
    pub external_link: Option<String>,
    pub royalty_info: Option<RoyaltyInfoResponse>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct TransferHookMsg {
    pub sender: String,
    pub recipient: String,
    pub token_id: String,
}

impl TransferHookMsg {
    /// serializes the message
    pub fn into_binary(self) -> StdResult<Binary> {
        let msg = TransferExecuteMsg::TransferHook(self);
        to_binary(&msg)
    }
}

// This is just a helper to properly serialize the above message
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum TransferExecuteMsg {
    TransferHook(TransferHookMsg),
}
