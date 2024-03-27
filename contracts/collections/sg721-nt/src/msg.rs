use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[allow(deprecated)]
use sg721::{RoyaltyInfoResponse, UpdateCollectionInfoMsg};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[allow(deprecated)]
pub enum ExecuteMsg<TNftExtensionMsg> {
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
        extension: TNftExtensionMsg,
    },
    /// Burn an NFT the sender has access to
    Burn { token_id: String },
    /// Update collection info
    #[allow(deprecated)]
    UpdateCollectionInfo {
        #[allow(deprecated)]
        new_collection_info: UpdateCollectionInfoMsg<RoyaltyInfoResponse>,
    },
    /// Freeze collection info from further updates
    FreezeCollectionInfo {},
}
