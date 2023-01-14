use cosmwasm_schema::cw_serde;
use sg_controllers::ContractInstantiateMsg;

#[cw_serde]
pub enum Group {
    Cw4Instantiate(ContractInstantiateMsg),
    Cw4Address(String),
}

#[cw_serde]
pub struct InstantiateMsg {
    pub group: Group,
}

#[cw_serde]
pub enum ExecuteMsg {
    Distribute {},
}

#[cw_serde]
pub enum QueryMsg {
    /// Returns the config
    Group {},
    /// Returns Member
    Member { address: String },
    /// Returns MemberListResponse
    ListMembers {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}
