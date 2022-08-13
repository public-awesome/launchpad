use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Saved in every minter
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MinterConfig<T> {
    pub factory: Addr,
    pub collection_code_id: u64,
    pub extension: T,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MinterConfigResponse<T> {
    pub config: MinterConfig<T>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Sg3QueryMsg {
    /// Returns `MinterConfigResponse<T>`
    Config {},
}
