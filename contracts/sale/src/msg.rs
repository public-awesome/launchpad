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
    pub whitelist_expiration: Option<Expiration>,
    pub whitelist_addresses: Option<Vec<String>>,
    pub start_time: Option<Expiration>,
    pub per_address_limit: Option<u64>,
    pub unit_price: Coin,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Mint {},
    UpdateWhitelist(UpdateWhitelistMsg),
    UpdateWhitelistExpiration(Expiration),
    UpdateStartTime(Expiration),
    UpdatePerAddressLimit { per_address_limit: u64 },
    MintFor { recipient: Addr },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetConfig {},
    // TODO other helpful queries
    // How many tokens are left in the sale?
    // List of token Uris?
    GetWhitelistAddresses {},
    GetWhitelistExpiration {},
    GetStartTime {},
    OnWhitelist { address: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateWhitelistMsg {
    pub add_addresses: Option<Vec<String>>,
    pub remove_addresses: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub admin: Addr,
    pub base_token_uri: String,
    pub num_tokens: u64,
    pub sg721_address: Addr,
    pub sg721_code_id: u64,
    pub unit_price: Coin,
    pub unused_token_id: u64,
    pub per_address_limit: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WhitelistAddressesResponse {
    pub addresses: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WhitelistExpirationResponse {
    pub expiration_time: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StartTimeResponse {
    pub start_time: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OnWhitelistResponse {
    pub on_whitelist: bool,
}
