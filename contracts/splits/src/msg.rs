use cosmwasm_schema::cw_serde;

use crate::state::{Config, Executor};

#[cw_serde]
pub struct InstantiateMsg {
    // this is the group contract that contains the member list
    pub group_addr: String,
    // Who is able to call distribute
    // None means that anyone can call distribute
    pub executor: Option<Executor>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Distribute {},
    UpdateExecutor { executor: Option<Executor> },
}

#[cw_serde]
pub enum SudoMsg {
    UpdateExecutor { executor: Option<Executor> },
}

#[cw_serde]
pub enum QueryMsg {
    /// Returns the config
    Config {},
    /// Returns Member
    Member { address: String },
    /// Returns MemberListResponse
    ListMembers {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[cw_serde]
pub struct ConfigResponse {
    pub config: Config,
}
