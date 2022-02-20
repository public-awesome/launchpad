use cw4::Member;
use cw_utils::Expiration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    // The minter contract is the only one that can update the whitelist
    pub minter: Option<String>,
    pub members: Vec<Member>,
    pub start_time: Expiration,
    pub end_time: Expiration,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateStartTime(Expiration),
    UpdateEndTime(Expiration),
    UpdateMembers {
        remove: Vec<String>,
        add: Vec<Member>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    StartTime {},
    EndTime {},
    ListMembers {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TimeResponse {
    pub time: String,
}
