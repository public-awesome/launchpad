use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, Timestamp};
use sg2::msg::{CreateMinterMsg, Sg2ExecuteMsg, UpdateMinterParamsMsg};

use crate::state::TokenMergeMinterParams;

#[cw_serde]
pub struct InstantiateMsg {
    pub params: TokenMergeMinterParams,
}

#[cw_serde]
pub struct TokenMergeMinterInitMsgExtension {
    pub base_token_uri: String,
    pub start_time: Timestamp,
    pub num_tokens: u32,
    pub mint_tokens: Vec<MintToken>,
    pub per_address_limit: u32,
}
pub type TokenMergeMinterCreateMsg = CreateMinterMsg<TokenMergeMinterInitMsgExtension>;

pub type ExecuteMsg = Sg2ExecuteMsg<TokenMergeMinterInitMsgExtension>;

#[cw_serde]
pub enum SudoMsg {
    UpdateParams(Box<TokenMergeUpdateParamsMsg>),
}

/// Message for params so they can be updated individually by governance
#[cw_serde]
pub struct TokenMergeUpdateParamsExtension {
    pub max_token_limit: Option<u32>,
    pub max_per_address_limit: Option<u32>,
    pub airdrop_mint_price: Option<Coin>,
    pub airdrop_mint_fee_bps: Option<u64>,
    pub shuffle_fee: Option<Coin>,
}
pub type TokenMergeUpdateParamsMsg = UpdateMinterParamsMsg<TokenMergeUpdateParamsExtension>;

#[cw_serde]
pub struct MintToken {
    pub collection: String,
    pub amount: u32,
}
#[cw_serde]
pub struct ParamsResponse {
    pub params: TokenMergeMinterParams,
}
