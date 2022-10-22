use cosmwasm_schema::cw_serde;

use crate::state::Config;

#[cw_serde]
pub struct InstantiateMsg {
    // this is the group contract that contains the member list
    pub group_addr: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Distribute {},
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
