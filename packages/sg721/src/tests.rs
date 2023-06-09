use crate::{CollectionInfo, RoyaltyInfo};
use cosmwasm_std::{Decimal, Timestamp};

pub fn mock_collection_info() -> CollectionInfo<RoyaltyInfo<String>> {
    CollectionInfo {
        creator: "creator".to_string(),
        description: String::from("Stargaze Monkeys"),
        image: "https://example.com/image.png".to_string(),
        external_link: Some("https://example.com/external.html".to_string()),
        royalty_info: Some(RoyaltyInfo {
            payment_address: "curator".to_string(),
            share: Decimal::percent(10),
            updated_at: Timestamp::from_nanos(0),
        }),
        start_trading_time: None,
        explicit_content: None,
    }
}
