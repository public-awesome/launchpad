use cosmwasm_std::{Addr, Coin};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_utils::Expiration;
use sg721::msg::InstantiateMsg as Sg721InstantiateMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub base_token_uri: String,
    pub num_tokens: u64,
    pub sg721_code_id: u64,
    pub sg721_instantiate_msg: Sg721InstantiateMsg,
    pub per_address_limit: Option<u64>,
    pub batch_mint_limit: Option<u64>,
    pub unit_price: Coin,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Mint {},
    SetWhitelist { whitelist: String },
    UpdatePerAddressLimit { per_address_limit: u64 },
    UpdateBatchMintLimit { batch_mint_limit: u64 },
    MintTo { recipient: Addr },
    MintFor { token_id: u64, recipient: Addr },
    BatchMint { num_mints: u64 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    MintableNumTokens {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub admin: Addr,
    pub base_token_uri: String,
    pub num_tokens: u64,
    pub sg721_address: Addr,
    pub sg721_code_id: u64,
    pub unit_price: Coin,
    pub per_address_limit: Option<u64>,
    pub batch_mint_limit: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MintableNumTokensResponse {
    pub count: u64,
}
