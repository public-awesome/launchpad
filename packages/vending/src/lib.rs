use cosmwasm_std::{Coin, Decimal, Timestamp, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sg721::{CollectionInfo, RoyaltyInfoResponse};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VendingMinterInitMsg {
    pub factory: String,
    pub sg721_code_id: u64,
    pub base_token_uri: String,
    pub name: String,
    pub symbol: String,
    pub collection_info: CollectionInfo<RoyaltyInfoResponse>,
    pub start_time: Timestamp,
    pub per_address_limit: u32,
    pub num_tokens: u32,
    pub unit_price: Coin,
    pub whitelist: Option<String>,
    pub params: VendingMinterParams,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Params {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VendingMinterParams {
    pub code_id: u64,
    pub max_token_limit: u32,
    pub max_per_address_limit: u32,
    pub min_mint_price: Uint128,
    pub airdrop_mint_price: Uint128,
    pub mint_fee_percent: Decimal,
    pub airdrop_mint_fee_percent: Decimal,
    pub creation_fee: Uint128,
    pub shuffle_fee: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SudoParams {
    pub minter_code_id: u64,
    pub vending_minter: VendingMinterParams,
}

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// pub struct MinterInfo<T> {
//     pub code_id: u64,
//     pub params: MinterParams<T>,
// }

// TODO: move to factory or minters package?
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ParamsResponse {
    pub params: SudoParams,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreateVendingMinter(VendingMinterInitMsg),
}

pub mod tests {
    use cosmwasm_std::{coin, Decimal, Timestamp, Uint128};
    use sg721::{CollectionInfo, RoyaltyInfoResponse};
    use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};

    use crate::{VendingMinterInitMsg, VendingMinterParams};

    pub const CREATION_FEE: u128 = 5_000_000_000;
    pub const MIN_MINT_PRICE: u128 = 50_000_000;
    pub const AIRDROP_MINT_PRICE: u128 = 15_000_000;
    pub const MINT_FEE_BPS: u64 = 1_000; // 10%
    pub const AIRDROP_MINT_FEE_BPS: u64 = 10_000; // 100%
    pub const SHUFFLE_FEE: u128 = 500_000_000;
    pub const MAX_TOKEN_LIMIT: u32 = 10000;
    pub const MAX_PER_ADDRESS_LIMIT: u32 = 50;

    pub fn mock_params() -> VendingMinterParams {
        VendingMinterParams {
            code_id: 1,
            max_token_limit: MAX_TOKEN_LIMIT,
            max_per_address_limit: MAX_PER_ADDRESS_LIMIT,
            min_mint_price: Uint128::from(MIN_MINT_PRICE),
            airdrop_mint_price: Uint128::from(AIRDROP_MINT_PRICE),
            mint_fee_percent: Decimal::percent(MINT_FEE_BPS),
            airdrop_mint_fee_percent: Decimal::percent(AIRDROP_MINT_FEE_BPS),
            creation_fee: Uint128::from(CREATION_FEE),
            shuffle_fee: Uint128::from(SHUFFLE_FEE),
        }
    }

    pub fn mock_init_msg() -> VendingMinterInitMsg {
        let collection_info: CollectionInfo<RoyaltyInfoResponse> = CollectionInfo {
            creator: "admin".to_string(),
            description: "description".to_string(),
            image: "https://example.com/image.png".to_string(),
            ..CollectionInfo::default()
        };

        VendingMinterInitMsg {
            num_tokens: 1,
            per_address_limit: 5,
            unit_price: coin(MIN_MINT_PRICE, NATIVE_DENOM),
            name: "Collection Name".to_string(),
            base_token_uri: "ipfs://test".to_string(),
            start_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME),
            sg721_code_id: 0,
            collection_info,
            factory: "factory".to_string(),
            symbol: "HAL".to_string(),
            whitelist: None,
            params: mock_params(),
        }
    }
}
