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
    pub start_time: Option<Expiration>,
    pub per_address_limit: Option<u32>,
    pub unit_price: Coin,
    pub whitelist: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Mint {},
    SetWhitelist { whitelist: String },
    UpdateStartTime(Expiration),
    UpdatePerAddressLimit { per_address_limit: u32 },
    MintTo { recipient: String },
    MintFor { token_id: u64, recipient: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    MintableNumTokens {},
    StartTime {},
    MintPrice {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub admin: Addr,
    pub base_token_uri: String,
    pub num_tokens: u64,
    pub per_address_limit: Option<u32>,
    pub sg721_address: Addr,
    pub sg721_code_id: u64,
    pub start_time: Option<Expiration>,
    pub unit_price: Coin,
    pub whitelist: Option<Addr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MintableNumTokensResponse {
    pub count: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StartTimeResponse {
    pub start_time: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MintPriceResponse {
    pub public_price: Coin,
    pub whitelist_price: Option<Coin>,
    pub current_price: Coin,
}
