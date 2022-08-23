use cosmwasm_std::{Addr, Coin};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Saved in every minter
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MinterConfig<T> {
    pub factory: Addr,
    pub collection_code_id: u64,
    pub mint_price: Coin,
    pub extension: T,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MinterConfigResponse<T> {
    pub config: MinterConfig<T>,
    pub collection_address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct Status {
    pub is_verified: bool,
    pub is_blocked: bool,
    pub is_explicit: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StatusResponse {
    pub status: Status,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Sg3QueryMsg {
    /// Returns `MinterConfigResponse<T>`
    Config {},
    /// Returns `StatusResponse`
    Status {},
}
