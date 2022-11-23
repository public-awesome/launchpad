use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::state::Config;

#[cw_serde]
pub struct InstantiateMsg {
    pub addresses: Vec<String>,
    pub per_address_limit: u32,
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
    IncludesAddress { address: String },
    #[returns(u64)]
    Admin {},
    #[returns(u64)]
    AddressCount {},
    #[returns(PerAddressLimitResponse)]
    PerAddressLimit {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub config: Config,
}

#[cw_serde]
pub struct PerAddressLimitResponse {
    pub limit: u64,
}
