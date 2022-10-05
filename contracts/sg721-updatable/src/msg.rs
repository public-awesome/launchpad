use cosmwasm_std::Binary;
use cw_utils::Expiration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sg721::ExecuteMsg as Sg721ExecuteMsg;
use sg721::MintMsg;
use sg721_base::msg::QueryMsg as Sg721QueryMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// freeze token metadata so creator can no longer update token uris
    FreezeTokenMetadata {},
    // creator calls can update token uris
    // UpdateTokenMetadata {
    //     token_id: String,
    //     token_uri: Option<String>,
    // },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {}
