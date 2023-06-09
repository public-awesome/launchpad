use crate::msg::CollectionParams;
use cosmwasm_std::Decimal;
use cosmwasm_std::Timestamp;
use sg721::tests::mock_collection_info;
use sg721::tests::mock_royalty_info;
use sg721::{CollectionInfo, RoyaltyInfo};

pub fn mock_collection_params() -> CollectionParams<String> {
    CollectionParams {
        code_id: 1,
        name: "Collection Name".to_string(),
        symbol: "COL".to_string(),
        info: CollectionInfo {
            start_trading_time: None,
            explicit_content: Some(false),
            royalty_info: Some(mock_royalty_info("creator".to_string())),
            ..mock_collection_info()
        },
    }
}

pub fn mock_collection_params_1(start_trading_time: Option<Timestamp>) -> CollectionParams<String> {
    CollectionParams {
        code_id: 1,
        name: "Collection Name".to_string(),
        symbol: "COL".to_string(),
        info: CollectionInfo {
            start_trading_time,
            explicit_content: Some(false),
            royalty_info: Some(mock_royalty_info("creator".to_string())),
            ..mock_collection_info()
        },
    }
}

pub fn mock_curator_payment_address(
    start_trading_time: Option<Timestamp>,
) -> CollectionParams<String> {
    CollectionParams {
        code_id: 1,
        name: String::from("Test Coin"),
        symbol: String::from("TEST"),
        info: CollectionInfo {
            royalty_info: Some(mock_royalty_info("curator".to_string())),
            start_trading_time,
            ..mock_collection_info()
        },
    }
}

pub fn mock_collection_params_high_fee(
    start_trading_time: Option<Timestamp>,
) -> CollectionParams<String> {
    CollectionParams {
        code_id: 1,
        name: String::from("Test Coin"),
        symbol: String::from("TEST"),
        info: CollectionInfo {
            royalty_info: Some(RoyaltyInfo {
                share: Decimal::percent(100),
                ..mock_royalty_info("creator".to_string())
            }),
            start_trading_time,
            ..mock_collection_info()
        },
    }
}

pub fn mock_collection_two(start_trading_time: Option<Timestamp>) -> CollectionParams<String> {
    CollectionParams {
        code_id: 1,
        name: String::from("Test Collection 2"),
        symbol: String::from("TEST 2"),
        info: CollectionInfo {
            creator: "creator".to_string(),
            description: String::from("Stargaze Monkeys 2"),
            image:
                "ipfs://bafybeigi3bwpvyvsmnbj46ra4hyffcxdeaj6ntfk5jpic5mx27x6ih2qvq/images/1.png"
                    .to_string(),
            external_link: Some("https://example.com/external.html".to_string()),
            royalty_info: Some(mock_royalty_info("creator".to_string())),
            start_trading_time,
            explicit_content: None,
            ..mock_collection_info()
        },
    }
}
