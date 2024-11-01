use crate::state::TokenMergeFactoryParams;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, Timestamp};
use sg721::{CollectionInfo, RoyaltyInfoResponse};

#[cw_serde]
pub struct InstantiateMsg {
    pub params: TokenMergeFactoryParams,
}

#[cw_serde]
pub struct TokenMergeMinterInitMsgExtension {
    pub base_token_uri: String,
    pub start_time: Timestamp,
    pub num_tokens: u32,
    pub mint_tokens: Vec<MintToken>,
    pub per_address_limit: u32,
}

#[cw_serde]
pub struct CollectionParams {
    /// The collection code id
    pub code_id: u64,
    pub name: String,
    pub symbol: String,
    pub info: CollectionInfo<RoyaltyInfoResponse>,
}
#[cw_serde]
pub struct CreateMinterMsg<T> {
    pub init_msg: T,
    pub collection_params: CollectionParams,
}

pub type TokenMergeMinterCreateMsg = CreateMinterMsg<TokenMergeMinterInitMsgExtension>;

#[cw_serde]
pub enum ExecuteMsg {
    CreateMinter(TokenMergeMinterCreateMsg),
}

#[cw_serde]
pub enum QueryMsg {
    Params {},
    AllowedCollectionCodeIds {},
    AllowedCollectionCodeId(u64),
}

#[cw_serde]
pub struct TokenMergeUpdateParamsExtension {
    pub max_token_limit: Option<u32>,
    pub max_per_address_limit: Option<u32>,
    pub airdrop_mint_price: Option<Coin>,
    pub airdrop_mint_fee_bps: Option<u64>,
    pub shuffle_fee: Option<Coin>,
}
#[cw_serde]
pub struct UpdateMinterParamsMsg<T> {
    /// The minter code id
    pub code_id: Option<u64>,
    pub add_sg721_code_ids: Option<Vec<u64>>,
    pub rm_sg721_code_ids: Option<Vec<u64>>,
    pub frozen: Option<bool>,
    pub creation_fee: Option<Coin>,
    pub max_trading_offset_secs: Option<u64>,
    pub extension: T,
}
pub type TokenMergeUpdateParamsMsg = UpdateMinterParamsMsg<TokenMergeUpdateParamsExtension>;

#[cw_serde]
pub enum SudoMsg {
    UpdateParams(Box<TokenMergeUpdateParamsMsg>),
}

#[cw_serde]
pub struct MintToken {
    pub collection: String,
    pub amount: u32,
}
#[cw_serde]
pub struct ParamsResponse {
    pub params: TokenMergeFactoryParams,
}

#[cw_serde]
pub struct AllowedCollectionCodeIdsResponse {
    pub code_ids: Vec<u64>,
}

#[cw_serde]
pub struct AllowedCollectionCodeIdResponse {
    pub allowed: bool,
}
