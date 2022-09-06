use cosmwasm_std::{Coin, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sg2::msg::{CreateMinterMsg, Sg2ExecuteMsg, UpdateMinterParamsMsg};

use crate::state::VendingMinterParams;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub params: VendingMinterParams,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct VendingMinterInitMsgExtension {
    pub base_token_uri: String,
    pub payment_address: Option<String>,
    pub start_time: Timestamp,
    pub num_tokens: u32,
    pub mint_price: Coin,
    pub per_address_limit: u32,
    pub whitelist: Option<String>,
}
pub type VendingMinterCreateMsg = CreateMinterMsg<VendingMinterInitMsgExtension>;

pub type ExecuteMsg = Sg2ExecuteMsg<VendingMinterInitMsgExtension>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SudoMsg {
    UpdateParams(Box<VendingUpdateParamsMsg>),
}

/// Message for params so they can be updated invidiually by governance
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct VendingUpdateParamsExtension {
    pub max_token_limit: Option<u32>,
    pub max_per_address_limit: Option<u32>,
    pub airdrop_mint_price: Option<Coin>,
    pub airdrop_mint_fee_bps: Option<u64>,
    pub shuffle_fee: Option<Coin>,
}
pub type VendingUpdateParamsMsg = UpdateMinterParamsMsg<VendingUpdateParamsExtension>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ParamsResponse {
    pub params: VendingMinterParams,
}
