use cosmwasm_std::{Addr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Delegate funds to a Stargaze validator
    Delegate {
        validator: String,
    },
    // Start the undelegation process
    Undelegate {
        validator: String,
        amount: Uint128,
    },
    // Redelegate {
    //     old_validator: String,
    //     new_validator: String,
    // },
    Claim {
        validator: String,
    },
    // Find all undelegations that have expired. Delete them from the map and send to the sender.
    // Withdraw {},
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
