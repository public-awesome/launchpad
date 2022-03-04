use cosmwasm_std::Coin;
use cw_utils::Expiration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub members: Vec<String>,
    pub start_time: Expiration,
    pub end_time: Expiration,
    pub unit_price: Coin,
    pub per_address_limit: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateStartTime(Expiration),
    UpdateEndTime(Expiration),
    UpdateMembers(AddMembersMsg),
    UpdatePerAddressLimit(u32),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AddMembersMsg {
    pub to_add: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    HasStarted {},
    HasEnded {},
    IsActive {},
    Members {},
    HasMember { member: String },
    Config {},
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct MembersResponse {
    pub members: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct HasMemberResponse {
    pub has_member: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TimeResponse {
    pub time: Expiration,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HasEndedResponse {
    pub has_ended: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HasStartedResponse {
    pub has_started: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IsActiveResponse {
    pub is_active: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UnitPriceResponse {
    pub unit_price: Coin,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub num_members: u32,
    pub per_address_limit: u32,
    pub start_time: Expiration,
    pub end_time: Expiration,
    pub unit_price: Coin,
}
