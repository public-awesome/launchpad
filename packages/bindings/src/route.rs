use schemars::JsonSchema;

use serde::{Deserialize, Serialize};

/// StargazeRoute is enum type to represent stargaze query route path
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StargazeRoute {
    Alloc,
    Claim,
    Distribution,
}
