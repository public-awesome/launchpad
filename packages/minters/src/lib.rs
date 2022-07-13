use cosmwasm_std::{Decimal, Uint128};
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
    pub code_id: u64,
    pub name: String,
    pub symbol: String,
    pub info: CollectionInfo<RoyaltyInfoResponse>,
}

/// Common params for all minters, updatable by governance
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MinterParams<T> {
    pub factory: String,
    pub code_id: u64,
    pub creation_fee: Uint128,
    pub max_token_limit: u32,
    pub max_per_address_limit: u32,
    pub min_mint_price: Uint128,
    pub airdrop_mint_price: Uint128,
    pub mint_fee_percent: Decimal,
    pub airdrop_mint_fee_percent: Decimal,
    pub extension: T,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Params {},
}

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// pub struct MinterInfo<T> {
//     pub code_id: u64,
//     pub params: MinterParams<T>,
// }

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ParamsResponse<T> {
    pub params: MinterParams<T>,
}

pub mod tests {
    use cosmwasm_std::Decimal;
    use sg721::{CollectionInfo, RoyaltyInfoResponse};

    use crate::CollectionParams;

    pub fn mock_collection_params() -> CollectionParams {
        CollectionParams {
            code_id: 1,
            name: "Collection Name".to_string(),
            symbol: "COL".to_string(),
            info: CollectionInfo {
                creator: "creator".to_string(),
                description: String::from("Stargaze Monkeys"),
                image: "https://example.com/image.png".to_string(),
                external_link: Some("https://example.com/external.html".to_string()),
                royalty_info: Some(RoyaltyInfoResponse {
                    payment_address: "creator".to_string(),
                    share: Decimal::percent(10),
                }),
            },
        }
    }
}
