use crate::{Minter, MinterParams};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Sg2QueryMsg {
    /// Returns `ParamsResponse`
    Params {},
    /// Returns a `MinterStatusResponse`
    MinterStatus { minter: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ParamsResponse<T> {
    pub params: MinterParams<T>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MinterStatusResponse {
    pub minter: Minter,
}
