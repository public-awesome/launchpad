use cosmwasm_std::{Addr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Delegate funds to a Stargaze validator.
    /// Also sets the withdraw address to the sender.
    /// `min_duration` is in days.
    Delegate {
        validator: String,
        min_duration: u64,
    },
    /// Starts the undelegation process
    Undelegate { validator: String, amount: Uint128 },
    /// Redelegate stake from one validator to another
    Redelegate {
        src_validator: String,
        dst_validator: String,
        amount: Uint128,
    },
    /// Claim rewards from a validator
    /// Rewards go to the withdraw address, not the contract.
    Claim { validator: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Return type: `DelegationsResponse`
    Delegations { address: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Delegation {
    pub validator: Addr,
    pub stake: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DelegationsResponse {
    pub delegations: Vec<Delegation>,
}
