use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, Timestamp};
use sg2::msg::{CreateMinterMsg, Sg2ExecuteMsg, UpdateMinterParamsMsg};

use crate::state::VendingMinterParams;

#[cw_serde]
pub struct InstantiateMsg {
    pub params: VendingMinterParams,
}

#[cw_serde]
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

#[cw_serde]
pub enum SudoMsg {
    UpdateParams(Box<VendingUpdateParamsMsg>),
}

/// Message for params so they can be updated individually by governance
#[cw_serde]
pub struct VendingUpdateParamsExtension {
    pub max_token_limit: Option<u32>,
    pub max_per_address_limit: Option<u32>,
    pub airdrop_mint_price: Option<Coin>,
    pub airdrop_mint_fee_bps: Option<u64>,
    pub shuffle_fee: Option<Coin>,
}
pub type VendingUpdateParamsMsg = UpdateMinterParamsMsg<VendingUpdateParamsExtension>;

#[cw_serde]
pub struct ParamsResponse {
    pub params: VendingMinterParams,
}
