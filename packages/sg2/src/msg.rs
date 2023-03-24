use cosmwasm_schema::cw_serde;
use cosmwasm_std::Coin;
use sg721::{CollectionInfo, RoyaltyInfoResponse};

#[cw_serde]
pub struct CreateMinterMsg<T> {
    pub init_msg: T,
    pub collection_params: CollectionParams,
}

#[cw_serde]
pub struct CollectionParams {
    /// The collection code id
    pub code_id: u64,
    pub name: String,
    pub symbol: String,
    pub info: CollectionInfo<RoyaltyInfoResponse>,
}

/// Message for params so they can be updated individually by governance
#[cw_serde]
pub struct UpdateMinterParamsMsg<T> {
    /// The minter code id
    pub code_id: Option<u64>,
    pub add_sg721_code_ids: Option<Vec<u64>>,
    pub rm_sg721_code_ids: Option<Vec<u64>>,
    pub frozen: Option<bool>,
    pub creation_fee: Option<Coin>,
    pub min_mint_price: Option<Coin>,
    pub mint_fee_bps: Option<u64>,
    pub max_trading_offset_secs: Option<u64>,
    pub extension: T,
}

#[cw_serde]
pub enum Sg2ExecuteMsg<T> {
    CreateMinter(CreateMinterMsg<T>),
}
