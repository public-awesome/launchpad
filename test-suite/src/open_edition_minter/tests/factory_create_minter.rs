use cosmwasm_std::{Coin, Timestamp, Uint128};
use open_edition_factory::state::ParamsExtension;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};

use crate::common_setup::setup_minter::common::constants::{
    DEV_ADDRESS, MAX_TOKEN_LIMIT, MIN_MINT_PRICE_OPEN_EDITION,
};
use crate::common_setup::setup_minter::open_edition_minter::minter_params::{
    default_nft_data, init_msg,
};
use crate::common_setup::templates::{
    open_edition_minter_custom_template, open_edition_minter_nft_data,
    open_edition_minter_start_and_end_time,
};
use open_edition_factory::types::{NftData, NftMetadataType};
use sg_metadata::{Metadata, Trait};

// let vt =
// open_edition_minter_custom_template(None, None, None, Some(10), Some(5), None, None, None);

#[test]
fn check_valid_create_minter() {
    // Set a per address lower or equal than the factory -> ok
    let max_per_address_limit = 10;
    let params_extension = ParamsExtension {
        max_token_limit: 10,
        max_per_address_limit,
        airdrop_mint_fee_bps: 100,
        airdrop_mint_price: Coin {
            denom: NATIVE_DENOM.to_string(),
            amount: Uint128::new(100_000_000u128),
        },
        dev_fee_address: DEV_ADDRESS.to_string(),
    };
    let per_address_limit_minter = Some(5);
    let init_msg = init_msg(
        default_nft_data(),
        per_address_limit_minter,
        None,
        Some(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10_000)),
        None,
        None,
    );
    let vt = open_edition_minter_custom_template(params_extension, init_msg).unwrap();
    assert!(vt.collection_response_vec[0].error.is_none())
}

#[test]
fn check_custom_denom_create_minter() {
    // Set a per address lower or equal than the factory -> ok
    let max_per_address_limit = 10;
    let params_extension = ParamsExtension {
        max_token_limit: 10,
        max_per_address_limit,
        airdrop_mint_fee_bps: 100,
        airdrop_mint_price: Coin {
            denom: "ibc/frenz".to_string(),
            amount: Uint128::new(100_000_000u128),
        },
        dev_fee_address: DEV_ADDRESS.to_string(),
    };
    let per_address_limit_minter = Some(5);
    let init_msg = init_msg(
        default_nft_data(),
        per_address_limit_minter,
        None,
        Some(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10_000)),
        None,
        Some(Coin {
            denom: "ibc/frenz".to_string(),
            amount: Uint128::new(100_000_000u128),
        }),
    );
    let vt = open_edition_minter_custom_template(params_extension, init_msg);
    assert!(vt.is_ok())
}

#[test]
fn check_invalid_create_minter_address_limit() {
    // If the absolute max per address defined in the factory is 10 and the message to init the
    // minter gives 20 -> error

    let max_per_address_limit = 10;
    let params_extension = ParamsExtension {
        max_token_limit: 10,
        max_per_address_limit,
        airdrop_mint_fee_bps: 100,
        airdrop_mint_price: Coin {
            denom: NATIVE_DENOM.to_string(),
            amount: Uint128::new(100_000_000u128),
        },
        dev_fee_address: DEV_ADDRESS.to_string(),
    };
    let per_address_limit_minter = Some(20);
    let init_msg_1 = init_msg(
        default_nft_data(),
        per_address_limit_minter,
        None,
        Some(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10_000)),
        None,
        None,
    );
    let vt = open_edition_minter_custom_template(params_extension.clone(), init_msg_1).unwrap();
    assert_eq!(
        vt.collection_response_vec[0]
            .error
            .as_ref()
            .unwrap()
            .root_cause()
            .to_string(),
        "Invalid minting limit per address. max: 10, min: 1, got: 20".to_string()
    );

    let per_address_limit_minter = Some(0);
    let init_msg_2 = init_msg(
        default_nft_data(),
        per_address_limit_minter,
        None,
        Some(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10_000)),
        None,
        None,
    );
    let vt = open_edition_minter_custom_template(params_extension, init_msg_2).unwrap();
    assert_eq!(
        vt.collection_response_vec[0]
            .error
            .as_ref()
            .unwrap()
            .root_cause()
            .to_string(),
        "Invalid minting limit per address. max: 10, min: 1, got: 0".to_string()
    );
}

#[test]
fn check_invalid_create_minter_start_end_time() {
    let params_extension = ParamsExtension {
        max_token_limit: 10,
        max_per_address_limit: 10,
        airdrop_mint_fee_bps: 100,
        airdrop_mint_price: Coin {
            denom: NATIVE_DENOM.to_string(),
            amount: Uint128::new(100_000_000u128),
        },
        dev_fee_address: DEV_ADDRESS.to_string(),
    };
    let per_address_limit_minter = Some(2);
    let start_time = Some(Timestamp::from_nanos(100_000));
    let init_msg_1 = init_msg(
        default_nft_data(),
        per_address_limit_minter,
        start_time,
        Some(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10_000)),
        None,
        None,
    );
    let vt = open_edition_minter_start_and_end_time(
        params_extension.clone(),
        init_msg_1,
        start_time,
        None,
    )
    .unwrap();
    assert_eq!(
        vt.collection_response_vec[0]
            .error
            .as_ref()
            .unwrap()
            .root_cause()
            .to_string(),
        "InvalidStartTime 0.000100000 < 1571797419.879305533"
    );

    let start_time = Some(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 100));
    let end_time = Some(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10));
    let init_msg_1 = init_msg(
        default_nft_data(),
        per_address_limit_minter,
        start_time,
        end_time,
        None,
        None,
    );
    let vt =
        open_edition_minter_start_and_end_time(params_extension, init_msg_1, start_time, end_time)
            .unwrap();

    assert_eq!(
        vt.collection_response_vec[0]
            .error
            .as_ref()
            .unwrap()
            .root_cause()
            .to_string(),
        "InvalidEndTime 1647032400.000000100 > 1647032400.000000010".to_string()
    );
}

#[test]
fn check_invalid_create_minter_mint_price() {
    // Invalid denom
    let params_extension = ParamsExtension {
        max_token_limit: 10,
        max_per_address_limit: 10,
        airdrop_mint_fee_bps: 100,
        airdrop_mint_price: Coin {
            denom: NATIVE_DENOM.to_string(),
            amount: Uint128::new(100_000_000u128),
        },
        dev_fee_address: DEV_ADDRESS.to_string(),
    };
    let per_address_limit_minter = Some(2);
    let init_msg_1 = init_msg(
        default_nft_data(),
        per_address_limit_minter,
        None,
        Some(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10_000)),
        None,
        Some(Coin {
            denom: "uinvalid".to_string(),
            amount: Uint128::new(MIN_MINT_PRICE_OPEN_EDITION),
        }),
    );
    let vt = open_edition_minter_custom_template(params_extension.clone(), init_msg_1).unwrap();
    assert_eq!(
        vt.collection_response_vec[0]
            .error
            .as_ref()
            .unwrap()
            .root_cause()
            .to_string(),
        "InvalidDenom"
    );
    // Invalid price
    let init_msg_2 = init_msg(
        default_nft_data(),
        per_address_limit_minter,
        None,
        Some(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10_000)),
        None,
        Some(Coin {
            denom: NATIVE_DENOM.to_string(),
            amount: Uint128::new(100u128),
        }),
    );
    let vt = open_edition_minter_custom_template(params_extension, init_msg_2).unwrap();
    assert_eq!(
        vt.collection_response_vec[0]
            .error
            .as_ref()
            .unwrap()
            .root_cause()
            .to_string(),
        "InvalidMintPrice"
    );
}

#[test]
fn check_invalid_create_minter_nft_data() {
    let params_extension = ParamsExtension {
        max_token_limit: 10,
        max_per_address_limit: 10,
        airdrop_mint_fee_bps: 100,
        airdrop_mint_price: Coin {
            denom: NATIVE_DENOM.to_string(),
            amount: Uint128::new(100_000_000u128),
        },
        dev_fee_address: DEV_ADDRESS.to_string(),
    };
    let per_address_limit_minter = Some(2);
    let start_time = Some(Timestamp::from_nanos(100_000));
    let nft_data_1 = NftData {
        nft_data_type: NftMetadataType::OffChainMetadata,
        extension: None,
        token_uri: None,
    };
    let init_msg_1 = init_msg(
        nft_data_1.clone(),
        per_address_limit_minter,
        start_time,
        Some(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10_000)),
        None,
        None,
    );

    let vt =
        open_edition_minter_nft_data(params_extension.clone(), init_msg_1, nft_data_1).unwrap();
    assert_eq!(
        vt.collection_response_vec[0]
            .error
            .as_ref()
            .unwrap()
            .root_cause()
            .to_string(),
        "InvalidNftDataProvided"
    );

    let metadata_def = Some(Metadata {
        image: Some("https://k3hinzdutnzbpmacmzv3nygeicm5klx5lizzk4txdsnlos4gztsa.arweave.net/Vs6G5HSbchewAmZrtuDEQJnVLv1aM5Vydxyat0uGzOQ".to_string()),
        image_data: None,
        external_url: Some("https://www.google.com".to_string()),
        description: Some("Description".to_string()),
        name: Some("name".to_string()),
        attributes: Some(vec![
            Trait {
                display_type: None,
                trait_type: "Hello".to_string(),
                value: "My Friend".to_string(),
            }
        ]),
        background_color: None,
        animation_url: None,
        youtube_url: None,
    });

    let nft_data_2 = NftData {
        nft_data_type: NftMetadataType::OffChainMetadata,
        extension: metadata_def,
        token_uri: None,
    };

    let init_msg_2 = init_msg(
        nft_data_2.clone(),
        per_address_limit_minter,
        start_time,
        Some(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10_000)),
        None,
        None,
    );

    let vt =
        open_edition_minter_nft_data(params_extension.clone(), init_msg_2, nft_data_2).unwrap();
    assert_eq!(
        vt.collection_response_vec[0]
            .error
            .as_ref()
            .unwrap()
            .root_cause()
            .to_string(),
        "InvalidNftDataProvided"
    );

    let token_uri_def = Some(
        "ipfs://bafybeigi3bwpvyvsmnbj46ra4hyffcxdeaj6ntfk5jpic5mx27x6ih2qvq/images/1.png"
            .to_string(),
    );

    let nft_data_3 = NftData {
        nft_data_type: NftMetadataType::OnChainMetadata,
        extension: None,
        token_uri: token_uri_def,
    };

    let init_msg_3 = init_msg(
        nft_data_3.clone(),
        per_address_limit_minter,
        start_time,
        Some(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10_000)),
        None,
        None,
    );

    let vt = open_edition_minter_nft_data(params_extension, init_msg_3, nft_data_3).unwrap();

    assert_eq!(
        vt.collection_response_vec[0]
            .error
            .as_ref()
            .unwrap()
            .root_cause()
            .to_string(),
        "InvalidNftDataProvided"
    );
}

#[test]
fn check_invalid_create_minter_max_tokens() {
    // Invalid max tokens
    let params_extension = ParamsExtension {
        max_token_limit: MAX_TOKEN_LIMIT,
        max_per_address_limit: 10,
        airdrop_mint_fee_bps: 100,
        airdrop_mint_price: Coin {
            denom: NATIVE_DENOM.to_string(),
            amount: Uint128::new(100_000_000u128),
        },
        dev_fee_address: DEV_ADDRESS.to_string(),
    };
    let per_address_limit_minter = Some(2);
    let init_msg_1 = init_msg(
        default_nft_data(),
        per_address_limit_minter,
        None,
        None,
        Some(MAX_TOKEN_LIMIT + 1),
        None,
    );
    let vt = open_edition_minter_custom_template(params_extension.clone(), init_msg_1).unwrap();
    assert!(vt.collection_response_vec[0].error.is_some());

    // Number of Tokens and End Time are both None
    let init_msg_2 = init_msg(
        default_nft_data(),
        per_address_limit_minter,
        None,
        None,
        None,
        None,
    );
    let vt = open_edition_minter_custom_template(params_extension, init_msg_2).unwrap();
    assert!(vt.collection_response_vec[0].error.is_some());
}
