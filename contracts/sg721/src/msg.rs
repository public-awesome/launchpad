use crate::state::{Extension, RoyaltyInfo};
use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub name: String,
    pub symbol: String,
    pub minter: String,
    pub extension: Extension,
}

// specialize ExecuteMsg with the CreatorInfo extention
pub type ExecuteMsg = cw721_base::ExecuteMsg<Extension>;

// The serde untagged attribute will remove both "base" and "extended" from the JSON, so we
// have the same effect as #[serde(flatten)] without using it, but have to add an additional
// Extended variant. It's still cleaner than copying over all the variants from base.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum QueryMsg {
    Base(cw721_base::QueryMsg),
    Extended(ExtendedQueryMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum ExtendedQueryMsg {
    Creator {},
    Royalties {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CreatorResponse {
    pub creator: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RoyaltyResponse {
    pub royalty: Option<RoyaltyInfo>,
}
