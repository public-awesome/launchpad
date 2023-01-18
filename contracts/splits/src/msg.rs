use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use sg_controllers::ContractInstantiateMsg;

#[cw_serde]
pub enum Group {
    Cw4Instantiate(ContractInstantiateMsg),
    Cw4Address(String),
}

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<String>,
    pub group: Group,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateAdmin { admin: Option<String> },
    Distribute {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(cw_controllers::AdminResponse)]
    Admin {},

    #[returns(Addr)]
    Group {},

    #[returns(cw4::MemberResponse)]
    Member { address: String },

    #[returns(cw4::MemberListResponse)]
    ListMembers {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}
