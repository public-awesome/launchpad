use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, Decimal};
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

impl Default for CollectionParams {
    fn default() -> Self {
        Self {
            code_id: 1,
            name: "Collection Name".to_string(),
            symbol: "COL".to_string(),
            info: CollectionInfo {
                creator: "creator".to_string(),
                description: String::from("Stargaze Monkeys"),
                image: "https://example.com/image.png".to_string(),
                external_link: Some("https://example.com/external.html".to_string()),
                start_trading_time: None,
                explicit_content: Some(false),
                royalty_info: Some(RoyaltyInfoResponse {
                    payment_address: "creator".to_string(),
                    share: Decimal::percent(10),
                }),
            },
        }
    }
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
