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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum QueryMsg {
    OwnerOf {
        token_id: String,
        include_expired: Option<bool>,
    },
    ApprovedForAll {
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
