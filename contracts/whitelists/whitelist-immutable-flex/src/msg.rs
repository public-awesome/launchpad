use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::state::Config;

#[cw_serde]
pub struct Member {
    pub address: String,
    pub mint_count: u32,
}
#[cw_serde]
pub struct InstantiateMsg {
    pub members: Vec<Member>,
    pub mint_discount_bps: Option<u64>,
}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(bool)]
    HasMember { address: String },
    #[returns(MemberResponse)]
    Member { address: String },
    #[returns(u64)]
    Admin {},
    #[returns(u64)]
    AddressCount {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub config: Config,
}

#[cw_serde]
pub struct MemberResponse {
    pub member: Member,
}
