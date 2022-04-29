use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sg_marketplace::msg::SaleFinalizedHookMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub marketplace_addr: Option<String>,
    pub admin: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    #[serde(rename = "claim_mint_nft")]
    ClaimMintNFT {
        minter_address: String,
    },
    SaleFinalizedHook(SaleFinalizedHookMsg),
    /// Change the admin
    UpdateAdmin {
        admin: Option<String>,
    },
}
