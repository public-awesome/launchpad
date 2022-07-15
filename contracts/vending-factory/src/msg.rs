use cosmwasm_std::{Coin, Uint128};
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MinterInitMsg {
    CreateMinter {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SudoMsg {
    UpdateParam {
        code_id: Option<u64>,
        creation_fee: Option<Coin>,
        max_token_limit: Option<u32>,
        max_per_address_limit: Option<u32>,
        min_mint_price: Option<Coin>,
        airdrop_mint_price: Option<Coin>,
        mint_fee_bps: Option<u64>,
        airdrop_mint_fee_bps: Option<u64>,
        shuffle_fee: Option<Coin>,
    },
    VerifyMinter {
        minter: String,
    },
    BlockMinter {
        minter: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Params {},
}
