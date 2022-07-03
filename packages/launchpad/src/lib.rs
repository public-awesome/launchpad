use cosmwasm_std::{Decimal, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Params {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VendingMinterParams {
    pub max_token_limit: u32,
    pub max_per_address_limit: u32,
    pub min_mint_price: Uint128,
    pub airdrop_mint_price: Uint128,
    pub mint_fee_percent: Decimal,
    pub airdrop_mint_fee_percent: Decimal,
    pub shuffle_fee: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SudoParams {
    /// A list of allowed minter code IDs
    pub minter_codes: Vec<u64>,
    pub vending_minter: VendingMinterParams,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ParamsResponse {
    pub params: SudoParams,
}
