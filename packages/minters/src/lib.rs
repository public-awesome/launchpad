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
    pub code_id: u64,
    pub name: String,
    pub symbol: String,
    pub info: CollectionInfo<RoyaltyInfoResponse>,
}

/// Common params for all minters used for storage
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MinterParams<T> {
    pub code_id: u64,
    pub creation_fee: Coin,
    pub min_mint_price: Coin,
    pub mint_fee_bps: u64,
    pub extension: T,
}

/// Message for params so they can be updated invidiually by governance
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct UpdateParamsMsg<T> {
    pub code_id: Option<u64>,
    pub creation_fee: Option<Coin>,
    pub min_mint_price: Option<Coin>,
    pub mint_fee_bps: Option<u64>,
    pub extension: T,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// Returns `ParamsResponse<T>`
pub enum QueryMsg {
    Params {},
    MinterStatus { minter: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ParamsResponse<T> {
    pub params: MinterParams<T>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Minter {
    pub verified: bool,
    pub blocked: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MinterStatusResponse {
    pub minter: Minter,
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
