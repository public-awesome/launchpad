use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: Addr,
    pub claim_msg_plaintext: String,
    pub airdrop_amount: u128,
    pub addresses: Vec<String>,
    pub whitelist_code_id: u64,
    pub minter_address: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct AirdropClaimResponse {
    result: bool,
    amount: u32,
    minter_page: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    ClaimAirdrop {
        eth_address: String,
        eth_sig: String,
    },
    AddEligibleEth {
        eth_addresses: Vec<String>,
    },
    UpdateMinterAddress {
        minter_address: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    AirdropEligible { eth_address: String },
    GetMinter {},
}

#[cw_serde]
pub struct VerifyResponse {
    pub verifies: bool,
}
