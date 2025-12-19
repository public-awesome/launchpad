use crate::msg::CollectionParams;
use cosmwasm_std::Timestamp;
use cw721::state::CollectionExtension;

// Note: royalty_info is set to None in test params because MockApi in cosmwasm-std 2.x
// requires valid bech32 addresses for validation. For royalty tests, use proper addresses.

pub fn mock_collection_params() -> CollectionParams {
    CollectionParams {
        code_id: 1,
        name: "Collection Name".to_string(),
        symbol: "COL".to_string(),
        creator: "creator".to_string(),
        info: CollectionExtension {
            description: String::from("Stargaze Monkeys"),
            image: "https://example.com/image.png".to_string(),
            external_link: Some("https://example.com/external.html".to_string()),
            start_trading_time: None,
            explicit_content: Some(false),
            royalty_info: None,
        },
    }
}

pub fn mock_collection_params_1(start_trading_time: Option<Timestamp>) -> CollectionParams {
    CollectionParams {
        code_id: 1,
        name: "Collection Name".to_string(),
        symbol: "COL".to_string(),
        creator: "creator".to_string(),
        info: CollectionExtension {
            description: String::from("Stargaze Monkeys"),
            image: "https://example.com/image.png".to_string(),
            external_link: Some("https://example.com/external.html".to_string()),
            start_trading_time,
            explicit_content: Some(false),
            royalty_info: None,
        },
    }
}

pub fn mock_curator_payment_address(start_trading_time: Option<Timestamp>) -> CollectionParams {
    CollectionParams {
        code_id: 1,
        name: String::from("Test Coin"),
        symbol: String::from("TEST"),
        creator: "creator".to_string(),
        info: CollectionExtension {
            description: String::from("Stargaze Monkeys"),
            image: "https://example.com/image.png".to_string(),
            external_link: Some("https://example.com/external.html".to_string()),
            royalty_info: None,
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
        creator: "creator".to_string(),
        info: CollectionExtension {
            description: String::from("Stargaze Monkeys"),
            image:
                "ipfs://bafybeigi3bwpvyvsmnbj46ra4hyffcxdeaj6ntfk5jpic5mx27x6ih2qvq/images/1.png"
                    .to_string(),
            external_link: Some("https://example.com/external.html".to_string()),
            royalty_info: None,
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
        creator: "creator".to_string(),
        info: CollectionExtension {
            description: String::from("Stargaze Monkeys 2"),
            image:
                "ipfs://bafybeigi3bwpvyvsmnbj46ra4hyffcxdeaj6ntfk5jpic5mx27x6ih2qvq/images/1.png"
                    .to_string(),
            external_link: Some("https://example.com/external.html".to_string()),
            royalty_info: None,
            start_trading_time,
            explicit_content: None,
        },
    }
}
