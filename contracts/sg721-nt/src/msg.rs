use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sg721::{CollectionInfo, MintMsg, RoyaltyInfoResponse};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg<T> {
    /// Called by the minter to put a collection contract in the ready state.
    /// When ready it means the factory and minter are properly setup.
    _Ready {},

    /// Mint a new NFT, can only be called by the contract minter
    Mint(MintMsg<T>),
    /// Burn an NFT the sender has access to
    Burn { token_id: String },
    /// Update collection info
    UpdateCollectionInfo {
        new_collection_info: CollectionInfo<RoyaltyInfoResponse>,
    },
    /// Freeze collection info from further updates
    FreezeCollectionInfo {},
}
