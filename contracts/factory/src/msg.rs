use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use minter::msg::InstantiateMsg as VendingMinterInitMsg;
use sg_std::StargazeMsgWrapper;

pub type Response = cosmwasm_std::Response<StargazeMsgWrapper>;
pub type SubMsg = cosmwasm_std::SubMsg<StargazeMsgWrapper>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub count: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MinterInitMsg {
    Increment {},
    Reset { count: i32 },
    CreateMinter {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Increment {},
    Reset { count: i32 },
    CreateVendingMinter(VendingMinterInitMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SudoMsg {
    VerifyMinter { minter: String },
    BlockMinter { minter: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    GetCount {},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CountResponse {
    pub count: i32,
}
