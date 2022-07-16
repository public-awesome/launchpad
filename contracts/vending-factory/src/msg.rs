use cosmwasm_std::Coin;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use sg_std::StargazeMsgWrapper;
use vending::VendingMinterParams;

pub type Response = cosmwasm_std::Response<StargazeMsgWrapper>;
pub type SubMsg = cosmwasm_std::SubMsg<StargazeMsgWrapper>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub params: VendingMinterParams,
}

/// Message for params so they can be updated invidiually by governance
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct UpdateParamsMsg {
    // TODO: move common params to minter package?
    pub code_id: Option<u64>,
    pub creation_fee: Option<Coin>,
    pub max_token_limit: Option<u32>,
    pub max_per_address_limit: Option<u32>,
    pub min_mint_price: Option<Coin>,
    pub airdrop_mint_price: Option<Coin>,
    pub mint_fee_bps: Option<u64>,
    pub airdrop_mint_fee_bps: Option<u64>,
    pub shuffle_fee: Option<Coin>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SudoMsg {
    UpdateParams(Box<UpdateParamsMsg>),
    UpdateMinterStatus {
        minter: String,
        verified: bool,
        blocked: bool,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Params {},
}
