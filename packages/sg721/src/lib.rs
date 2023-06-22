use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Decimal, Timestamp};
use cw_address_like::AddressLike;
use cw_ownable::cw_ownable_execute;
use cw_utils::Expiration;
use semver::Version;
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod tests;

#[cw_ownable_execute]
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
        extension: T,
    },
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
        collection_info: UpdateCollectionInfoMsg<RoyaltyInfo<String>>,
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
pub struct RoyaltyInfo<T: AddressLike> {
    pub payment_address: T,
    pub share: Decimal,
    // ref: https://serde.rs/attr-skip-serializing.html#skip-serializing-field
    #[serde(skip_serializing_if = "is_contract_version_less_than_3")]
    pub updated_at: Timestamp,
}

fn is_contract_version_less_than_3(_: &Timestamp) -> bool {
    match Version::parse(CONTRACT_VERSION) {
        Ok(current_version) => current_version < Version::new(2, 3, 0),
        Err(_) => true,
    }
}

impl From<RoyaltyInfo<Addr>> for RoyaltyInfo<String> {
    fn from(royalty_info: RoyaltyInfo<Addr>) -> Self {
        RoyaltyInfo {
            payment_address: royalty_info.payment_address.to_string(),
            share: royalty_info.share,
            updated_at: royalty_info.updated_at,
        }
    }
}

#[cw_serde]
pub struct InstantiateMsg {
    pub name: String,
    pub symbol: String,
    pub minter: String,
    pub collection_info: CollectionInfo<RoyaltyInfo<String>>,
}
