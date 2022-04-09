use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Sg2SudoMsg {
    UpdateParamu32 {
        contract_name: String,
        key: String,
        value: u32,
    },
}
