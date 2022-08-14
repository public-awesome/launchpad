use cw3_flex_multisig::state::Executor;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    // this is the group contract that contains the member list
    pub group_addr: String,
    // who is able to execute passed proposals
    // None means that anyone can execute
    pub executor: Option<Executor>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Distribute {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns VoterInfo
    Member { address: String },
    /// Returns VoterListResponse
    ListMembers {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}
