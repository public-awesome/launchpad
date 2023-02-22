use cw721_base::MintMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sg721::{RoyaltyInfoResponse, UpdateCollectionInfoMsg};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg<T> {
    /// Mint a new NFT, can only be called by the contract minter
    Mint(MintMsg<T>),
    /// Burn an NFT the sender has access to
    Burn { token_id: String },
    /// Update collection info
    UpdateCollectionInfo {
        new_collection_info: UpdateCollectionInfoMsg<RoyaltyInfoResponse>,
    },
    /// Freeze collection info from further updates
    FreezeCollectionInfo {},
}
