use cosmwasm_std::{Addr, Timestamp, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Delegate funds to a Stargaze validator.
/// `min_duration` is in days.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub validator: String,
    pub min_duration: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Starts the undelegation process
    Undelegate {},
    /// Redelegate stake from one validator to another
    Redelegate { dst_validator: String },
    /// Claim rewards from validator
    Claim {},
    /// Withdraw rewards and delegate to the validator
    Reinvest {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Return type: `DelegationsResponse`
    Delegations {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Delegation {
    pub validator: Addr,
    pub stake: Uint128,
    pub end_time: Timestamp,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DelegationsResponse {
    pub delegations: Vec<Delegation>,
}
