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
    pub group: Group,
}

#[cw_serde]
pub enum ExecuteMsg {
    Distribute {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
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
