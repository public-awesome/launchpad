use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Sg2QueryMsg {
    /// Gets a param by key
    /// Return type: `ParamResponse`
    GetParamu32 { contract_name: String, key: String },
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ParamResponseu32 {
    pub value: u32,
}
