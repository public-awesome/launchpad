use crate::state::Config;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub config: Config,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct EligibleResponse {
    pub eligible: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AirdropClaimResponse {
    result: bool,
    amount: u32,
    minter_page: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    ClaimAirdrop {
        eth_address: String,
        eth_sig: String,
        stargaze_address: String,
        stargaze_sig: String,
    },
    AddEligibleEth {
        eth_address: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    AirdropEligible { eth_address: Addr },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AirdropEligibleResponse {
    pub eligible: bool,
}

#[cw_serde]
pub struct VerifyResponse {
    pub verifies: bool,
}
