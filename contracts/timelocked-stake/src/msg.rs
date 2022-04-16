use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::Stake;

/// Delegate funds to a Stargaze validator.
/// `min_duration` is in days.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub validator: String,
    pub min_duration: u64,
    /// This is the minimum amount we will pull out to reinvest + claim
    pub min_withdrawal: Uint128,
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
    /// Private message to delegate as part of `Reinvest {}`
    _Delegate {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Return type: `StakeResponse`
    Stake {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakeResponse {
    pub stake: Stake,
}
