use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Coin, Timestamp};
use cw721_base::{
    msg::{
        CollectionMetadataExtensionMsg, CollectionMetadataMsg, ExecuteMsg as Cw721ExecuteMsg, InstantiateMsg as Cw721InstantiateMsg
    },
    DefaultOptionCollectionMetadataExtensionMsg,
};
use cw_ownable::Action;
use cw_utils::Expiration;

pub type RoyaltyInfoResponse = cw721_base::msg::RoyaltyInfoResponse;
pub use cw721_base::state::RoyaltyInfo;

#[cw_serde]
pub enum ExecuteMsg<
    // Message passed for updating metadata.
    TNftMetadataExtensionMsg,
    // Message passed for updating collection info extension.
    TCollectionMetadataExtensionMsg,
> {
    // ---- sg721 specific msgs ----
    /// Update specific collection info fields
    #[deprecated = "Please use UpdateCollectionInfo instead"]
    UpdateCollectionInfo {
        #[allow(deprecated)]
        collection_info: UpdateCollectionInfoMsg<RoyaltyInfoResponse>,
    },
    /// Called by the minter to update trading start time
    UpdateStartTradingTime(Option<Timestamp>),
    // Freeze collection info from further updates
    FreezeCollectionInfo,

    // ---- cw721 v0.19.0 msgs ----
    #[deprecated(since = "0.19.0", note = "Please use UpdateMinterOwnership instead")]
    /// Deprecated: use UpdateMinterOwnership instead! Will be removed in next release!
    UpdateOwnership(Action),
    UpdateMinterOwnership(Action),
    UpdateCreatorOwnership(Action),

    /// The creator is the only one eligible to update `CollectionMetadata`.
    UpdateCollectionMetadata {
        collection_metadata: CollectionMetadataMsg<TCollectionMetadataExtensionMsg>,
    },
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
    Mint {
        /// Unique ID of the NFT
        token_id: String,
        /// The owner of the newly minter NFT
        owner: String,
        /// Universal resource identifier for this NFT
        /// Should point to a JSON file that conforms to the ERC721
        /// Metadata JSON Schema
        token_uri: Option<String>,
        /// Any custom extension used by this contract
        extension: TNftMetadataExtensionMsg,
    },

    /// Burn an NFT the sender has access to
    Burn {
        token_id: String,
    },

    /// Metadata msg
    #[deprecated(since = "0.19.0", note = "Please use UpdateNftMetadata instead")]
    /// Deprecated: use UpdateNftMetadata instead! In previous release it was a no-op for customization in other contracts. Will be removed in next release!
    Extension {
        msg: TNftMetadataExtensionMsg,
    },
    /// The creator is the only one eligible to update NFT's token uri and onchain metadata (`NftInfo.extension`).
    /// NOTE: approvals and owner are not affected by this call, since they belong to the NFT owner.
    UpdateNftInfo {
        token_id: String,
        token_uri: Option<String>,
        extension: TNftMetadataExtensionMsg,
    },

    /// Sets address to send withdrawn fees to. Only owner can call this.
    SetWithdrawAddress {
        address: String,
    },
    /// Removes the withdraw address, so fees are sent to the contract. Only owner can call this.
    RemoveWithdrawAddress {},
    /// Withdraw from the contract to the given address. Anyone can call this,
    /// which is okay since withdraw address has been set by owner.
    WithdrawFunds {
        amount: Coin,
    },
}

impl<TNftMetadataExtensionMsg, TCollectionMetadataExtensionMsg>
    From<
        ExecuteMsg<
            TNftMetadataExtensionMsg,
            TCollectionMetadataExtensionMsg,
        >,
    >
    for Cw721ExecuteMsg<
        TNftMetadataExtensionMsg,
        TCollectionMetadataExtensionMsg,
    >
{
    #[allow(deprecated)]
    fn from(
        msg: ExecuteMsg<
            TNftMetadataExtensionMsg,
            TCollectionMetadataExtensionMsg,
        >,
    ) -> Self {
        match msg {
            // ---- sg721 msgs ----
            ExecuteMsg::UpdateCollectionInfo { collection_info: _ } => {
                panic!("not a cw721 msg")
            }
            ExecuteMsg::UpdateStartTradingTime(_) => panic!("not a cw721 msg"),
            ExecuteMsg::FreezeCollectionInfo => panic!("not a cw721 msg"),
            // ---- cw721 msgs ----
            ExecuteMsg::UpdateOwnership(action) => Cw721ExecuteMsg::UpdateOwnership(action),
            ExecuteMsg::UpdateMinterOwnership(action) => {
                Cw721ExecuteMsg::UpdateMinterOwnership(action)
            }
            ExecuteMsg::UpdateCreatorOwnership(action) => {
                Cw721ExecuteMsg::UpdateCreatorOwnership(action)
            }
            ExecuteMsg::UpdateCollectionMetadata {
                collection_metadata,
            } => Cw721ExecuteMsg::UpdateCollectionMetadata {
                collection_metadata,
            },
            ExecuteMsg::TransferNft {
                recipient,
                token_id,
            } => Cw721ExecuteMsg::TransferNft {
                recipient,
                token_id,
            },
            ExecuteMsg::SendNft {
                contract,
                token_id,
                msg,
            } => Cw721ExecuteMsg::SendNft {
                contract,
                token_id,
                msg,
            },
            ExecuteMsg::Approve {
                spender,
                token_id,
                expires,
            } => Cw721ExecuteMsg::Approve {
                spender,
                token_id,
                expires,
            },
            ExecuteMsg::Revoke { spender, token_id } => {
                Cw721ExecuteMsg::Revoke { spender, token_id }
            }
            ExecuteMsg::ApproveAll { operator, expires } => {
                Cw721ExecuteMsg::ApproveAll { operator, expires }
            }
            ExecuteMsg::RevokeAll { operator } => Cw721ExecuteMsg::RevokeAll { operator },
            ExecuteMsg::Mint {
                token_id,
                owner,
                token_uri,
                extension,
            } => Cw721ExecuteMsg::Mint {
                token_id,
                owner,
                token_uri,
                extension,
            },
            ExecuteMsg::Burn { token_id } => Cw721ExecuteMsg::Burn { token_id },
            ExecuteMsg::Extension { msg } => Cw721ExecuteMsg::Extension { msg },
            ExecuteMsg::UpdateNftInfo {
                token_id,
                token_uri,
                extension,
            } => Cw721ExecuteMsg::UpdateNftInfo {
                token_id,
                token_uri,
                extension,
            },
            ExecuteMsg::SetWithdrawAddress { address } => {
                Cw721ExecuteMsg::SetWithdrawAddress { address }
            }
            ExecuteMsg::RemoveWithdrawAddress {} => Cw721ExecuteMsg::RemoveWithdrawAddress {},
            ExecuteMsg::WithdrawFunds { amount } => Cw721ExecuteMsg::WithdrawFunds { amount },
        }
    }
}

#[cw_serde]
#[deprecated = "Please use CollectionMetadata instead"]
pub struct CollectionInfo<T> {
    pub creator: String,
    pub description: String,
    pub image: String,
    pub external_link: Option<String>,
    pub explicit_content: Option<bool>,
    pub start_trading_time: Option<Timestamp>,
    pub royalty_info: Option<T>,
}

#[allow(deprecated)]
impl From<CollectionInfo<RoyaltyInfoResponse>> for DefaultOptionCollectionMetadataExtensionMsg {
    fn from(info: CollectionInfo<RoyaltyInfoResponse>) -> Self {
        Some(CollectionMetadataExtensionMsg {
            description: Some(info.description),
            image: Some(info.image),
            external_link: info.external_link,
            explicit_content: info.explicit_content,
            start_trading_time: info.start_trading_time,
            royalty_info: info.royalty_info,
        })
    }
}

#[cw_serde]
#[deprecated = "Please use `UpdateCollectionMetadata<DefaultOptionCollectionMetadataExtensionMsg>` instead"]
pub struct UpdateCollectionInfoMsg<T> {
    pub description: Option<String>,
    pub image: Option<String>,
    pub external_link: Option<Option<String>>,
    pub explicit_content: Option<bool>,
    pub royalty_info: Option<Option<T>>,
    /// creator is ignore here, use `UpdateCreatorOwnership` instead
    pub creator: Option<String>,
}

#[allow(deprecated)]
impl From<UpdateCollectionInfoMsg<RoyaltyInfoResponse>> for CollectionMetadataExtensionMsg<RoyaltyInfoResponse> {
    fn from(msg: UpdateCollectionInfoMsg<RoyaltyInfoResponse>) -> Self {
        CollectionMetadataExtensionMsg {
            description: msg.description,
            image: msg.image,
            external_link: msg.external_link.unwrap_or_default(),
            explicit_content: msg.explicit_content,
            royalty_info: msg.royalty_info.unwrap_or_default(),
            start_trading_time: None,
        }
    }
}

#[cw_serde]
pub struct InstantiateMsg {
    pub name: String,
    pub symbol: String,
    pub minter: String,
    #[allow(deprecated)]
    pub collection_info: CollectionInfo<RoyaltyInfoResponse>,
}

#[allow(deprecated)]
impl From<InstantiateMsg> for Cw721InstantiateMsg<DefaultOptionCollectionMetadataExtensionMsg> {
    fn from(msg: InstantiateMsg) -> Self {
        Cw721InstantiateMsg {
            name: msg.name,
            symbol: msg.symbol,
            minter: Some(msg.minter),
            creator: Some(msg.collection_info.creator.clone()),
            collection_metadata_extension: msg.collection_info.into(),
            withdraw_address: None,
        }
    }
}
