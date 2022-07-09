use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use sg_std::StargazeMsgWrapper;
use vending::SudoParams;

pub type Response = cosmwasm_std::Response<StargazeMsgWrapper>;
pub type SubMsg = cosmwasm_std::SubMsg<StargazeMsgWrapper>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    // TODO: expand?
    pub params: SudoParams,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MinterInitMsg {
    CreateMinter {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SudoMsg {
    //TODO: UpdateParam {}
    VerifyMinter { minter: String },
    BlockMinter { minter: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Params {},
}
