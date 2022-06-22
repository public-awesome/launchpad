use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sg_marketplace::msg::SaleHookMsg;

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
    SaleHook(SaleHookMsg),
    /// Change or clear the admin
    UpdateAdmin {
        admin: Option<String>,
    },
    /// Only the admin can update the marketplace address
    UpdateMarketplace {
        marketplace_addr: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Return MarketplaceResponse
    Marketplace {},
    /// Return AdminResponse
    Admin {},
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct MarketplaceResponse {
    pub marketplace: Option<Addr>,
}
