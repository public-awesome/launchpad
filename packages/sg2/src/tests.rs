use crate::msg::CollectionParams;
use cosmwasm_std::Decimal;
use cosmwasm_std::Timestamp;
#[allow(deprecated)]
use sg721::{CollectionInfo, RoyaltyInfoResponse};

#[allow(deprecated)]
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
            explicit_content: Some(false),
            royalty_info: Some(RoyaltyInfoResponse {
                payment_address: "creator".to_string(),
                share: Decimal::percent(10),
            }),
        },
    }
}

/// `v3.8.0-prerelease` is only used for testing migration from collection info (sg721) to new collection extension (cw721)
#[allow(deprecated)]
pub fn mock_collection_params_v3_8_0_prerelease() -> sg2_v3_8_0_prerelease::msg::CollectionParams {
    sg2_v3_8_0_prerelease::msg::CollectionParams {
        code_id: 1,
        name: "Collection Name".to_string(),
        symbol: "COL".to_string(),
        info: sg721_v3_8_0_prerelease::CollectionInfo {
            creator: "creator".to_string(),
            description: String::from("Stargaze Monkeys"),
            image: "https://example.com/image.png".to_string(),
            external_link: Some("https://example.com/external.html".to_string()),
            start_trading_time: None,
            explicit_content: Some(false),
            royalty_info: Some(sg721_v3_8_0_prerelease::RoyaltyInfoResponse {
                payment_address: "creator".to_string(),
                share: Decimal::percent(10),
            }),
        },
    }
}

pub fn mock_collection_params_1(start_trading_time: Option<Timestamp>) -> CollectionParams {
    CollectionParams {
        code_id: 1,
        name: "Collection Name".to_string(),
        symbol: "COL".to_string(),
        #[allow(deprecated)]
        info: CollectionInfo {
            creator: "creator".to_string(),
            description: String::from("Stargaze Monkeys"),
            image: "https://example.com/image.png".to_string(),
            external_link: Some("https://example.com/external.html".to_string()),
            start_trading_time,
            explicit_content: Some(false),
            royalty_info: Some(RoyaltyInfoResponse {
                payment_address: "creator".to_string(),
                share: Decimal::percent(10),
            }),
        },
    }
}

pub fn mock_curator_payment_address(start_trading_time: Option<Timestamp>) -> CollectionParams {
    CollectionParams {
        code_id: 1,
        name: String::from("Test Coin"),
        symbol: String::from("TEST"),
        #[allow(deprecated)]
        info: CollectionInfo {
            creator: "creator".to_string(),
            description: String::from("Stargaze Monkeys"),
            image: "https://example.com/image.png".to_string(),
            external_link: Some("https://example.com/external.html".to_string()),
            royalty_info: Some(RoyaltyInfoResponse {
                payment_address: "curator".to_string(),
                share: Decimal::percent(10),
            }),
            start_trading_time,
            explicit_content: None,
        },
    }
}

pub fn mock_collection_params_high_fee(start_trading_time: Option<Timestamp>) -> CollectionParams {
    CollectionParams {
        code_id: 1,
        name: String::from("Test Coin"),
        symbol: String::from("TEST"),
        #[allow(deprecated)]
        info: CollectionInfo {
            creator: "creator".to_string(),
            description: String::from("Stargaze Monkeys"),
            image:
                "ipfs://bafybeigi3bwpvyvsmnbj46ra4hyffcxdeaj6ntfk5jpic5mx27x6ih2qvq/images/1.png"
                    .to_string(),
            external_link: Some("https://example.com/external.html".to_string()),
            royalty_info: Some(RoyaltyInfoResponse {
                payment_address: "creator".to_string(),
                share: Decimal::percent(100),
            }),
            start_trading_time,
            explicit_content: None,
        },
    }
}

pub fn mock_collection_two(start_trading_time: Option<Timestamp>) -> CollectionParams {
    CollectionParams {
        code_id: 1,
        name: String::from("Test Collection 2"),
        symbol: String::from("TEST 2"),
        #[allow(deprecated)]
        info: CollectionInfo {
            creator: "creator".to_string(),
            description: String::from("Stargaze Monkeys 2"),
            image:
                "ipfs://bafybeigi3bwpvyvsmnbj46ra4hyffcxdeaj6ntfk5jpic5mx27x6ih2qvq/images/1.png"
                    .to_string(),
            external_link: Some("https://example.com/external.html".to_string()),
            royalty_info: Some(RoyaltyInfoResponse {
                payment_address: "creator".to_string(),
                share: Decimal::percent(10),
            }),
            start_trading_time,
            explicit_content: None,
        },
    }
}
