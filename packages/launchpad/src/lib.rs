use cosmwasm_std::{Coin, Decimal, Timestamp, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use sg721::{CollectionInfo, RoyaltyInfoResponse};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct MinterInitMsg<T> {
    pub factory: String,
    pub sg721_code_id: u64,
    pub name: String,
    pub symbol: String,
    pub collection_info: CollectionInfo<RoyaltyInfoResponse>,
    pub start_time: Timestamp,
    pub per_address_limit: u32,
    pub unit_price: Coin,
    pub whitelist: Option<String>,
    pub max_token_limit: u32,
    pub min_mint_price: Uint128,
    pub airdrop_mint_price: Uint128,
    pub mint_fee_bps: u64,
    pub airdrop_mint_fee_bps: u64,
    pub extension: T,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct VendingMinterInitMsgExtension {
    pub shuffle_fee: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Params {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct MinterParams<T> {
    pub code_id: u64,
    pub max_token_limit: u32,
    pub max_per_address_limit: u32,
    pub min_mint_price: Uint128,
    pub airdrop_mint_price: Uint128,
    pub mint_fee_percent: Decimal,
    pub airdrop_mint_fee_percent: Decimal,
    pub creation_fee: Uint128,
    pub extension: T,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct VendingMinterParamsExtension {
    pub shuffle_fee: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SudoParams<T> {
    pub minters: Vec<MinterInfo<T>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MinterInfo<T> {
    pub code_id: u64,
    pub params: MinterParams<T>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ParamsResponse<T> {
    pub params: SudoParams<T>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg<T> {
    CreateMinter(MinterInitMsg<T>),
}
