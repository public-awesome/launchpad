use cosmwasm_std::Decimal;
use sg721::{CollectionInfo, RoyaltyInfoResponse};

use crate::msg::CollectionParams;

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
            start_trading_time: None,
            royalty_info: Some(RoyaltyInfoResponse {
                payment_address: "creator".to_string(),
                share: Decimal::percent(10),
            }),
        },
    }
}
