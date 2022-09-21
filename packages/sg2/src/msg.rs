use cosmwasm_std::Coin;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sg721::{CollectionInfo, RoyaltyInfoResponse};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CreateMinterMsg<T> {
    pub init_msg: T,
    pub collection_params: CollectionParams,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollectionParams {
    /// The collection code id
    pub code_id: u64,
    pub name: String,
    pub symbol: String,
    pub info: CollectionInfo<RoyaltyInfoResponse>,
}

/// Message for params so they can be updated invidiually by governance
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct UpdateMinterParamsMsg<T> {
    /// The minter code id
    pub code_id: Option<u64>,
    pub creation_fee: Option<Coin>,
    pub min_mint_price: Option<Coin>,
    pub mint_fee_bps: Option<u64>,
    pub max_trading_offset_secs: Option<u64>,
    pub extension: T,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Sg2ExecuteMsg<T> {
    CreateMinter(CreateMinterMsg<T>),
}
