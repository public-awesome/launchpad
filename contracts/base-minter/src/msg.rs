use base_factory::{msg::BaseMinterCreateMsg, state::BaseMinterParams};
use cosmwasm_std::Empty;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sg3::MinterConfigResponse;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub create_msg: BaseMinterCreateMsg,
    pub params: BaseMinterParams,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Mint { token_uri: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
}

pub type ConfigResponse = MinterConfigResponse<Empty>;
