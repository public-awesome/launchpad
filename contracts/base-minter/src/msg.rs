use base_factory::{msg::BaseMinterCreateMsg, state::BaseMinterParams};
use cosmwasm_std::Timestamp;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sg4::MinterConfigResponse;

use crate::state::ConfigExtension;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub create_msg: BaseMinterCreateMsg,
    pub params: BaseMinterParams,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Mint {
        token_uri: String,
    },
    UpdateStartTradingTime {
        start_trading_time: Option<Timestamp>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Status {},
}

pub type ConfigResponse = MinterConfigResponse<ConfigExtension>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SudoMsg {
    UpdateStatus {
        is_verified: bool,
        is_blocked: bool,
        is_explicit: bool,
    },
}
