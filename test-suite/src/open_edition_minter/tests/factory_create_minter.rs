use cosmwasm_std::{Coin, Timestamp, Uint128};
use open_edition_factory::ContractError as OpenEditionContractError;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};

use open_edition_factory::types::{NftData, NftMetadataType};
use sg_metadata::{Metadata, Trait};

use crate::common_setup::setup_minter::common::constants::MIN_MINT_PRICE_OPEN_EDITION;
use crate::common_setup::templates::open_edition_minter_custom_template;

#[test]
fn check_valid_create_minter() {
    // Set a per address lower or equal than the factory -> ok
    let vt = open_edition_minter_custom_template(
        None,
        None,
        None,
        Some(10),
        Some(5),
        None,
        None,
        None,
        None,
    );
    assert!(vt.is_ok());
}

#[test]
fn check_invalid_create_minter_address_limit() {
    // If the absolute max per address defined in the factory is 10 and the message to init the
    // minter gives 20 -> error
    let vt = open_edition_minter_custom_template(
        None,
        None,
        None,
        Some(10),
        Some(20),
        None,
        None,
        None,
        None,
    );
    // When it is an error -> wrapped twice
    assert_eq!(
        vt.err()
            .unwrap()
            .err()
            .unwrap()
            .source()
            .unwrap()
            .to_string(),
        OpenEditionContractError::InvalidPerAddressLimit {
            max: 10,
            min: 1,
            got: 20
        }
        .to_string()
    );

    // The minimum should be 1 -> 0 will give an error
    let vt = open_edition_minter_custom_template(
        None,
        None,
        None,
        Some(10),
        Some(0),
        None,
        None,
        None,
        None,
    );
    assert_eq!(
        vt.err()
            .unwrap()
            .err()
            .unwrap()
            .source()
            .unwrap()
            .to_string(),
        OpenEditionContractError::InvalidPerAddressLimit {
            max: 10,
            min: 1,
            got: 0
        }
        .to_string()
    );
}

#[test]
fn check_invalid_create_minter_start_end_time() {
    // If start time < now
    let vt = open_edition_minter_custom_template(
        Some(Timestamp::from_nanos(100_000)),
        None,
        None,
        Some(10),
        Some(2),
        None,
        None,
        None,
        None,
    );
    assert_eq!(
        vt.err()
            .unwrap()
            .err()
            .unwrap()
            .source()
            .unwrap()
            .to_string(),
        "InvalidStartTime 0.000100000 < 1571797419.879305533".to_string()
    );

    // If start time > end time
    let vt = open_edition_minter_custom_template(
        Some(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 100)),
        Some(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10)),
        None,
        Some(10),
        Some(2),
        None,
        None,
        None,
        None,
    );
    assert_eq!(
        vt.err()
            .unwrap()
            .err()
            .unwrap()
            .source()
            .unwrap()
            .to_string(),
        "InvalidEndTime 1647032400.000000100 > 1647032400.000000010".to_string()
    );
}

#[test]
fn check_invalid_create_minter_mint_price() {
    // Invalid denom
    let vt = open_edition_minter_custom_template(
        None,
        None,
        None,
        Some(10),
        Some(2),
        Some(Coin {
            denom: "uinvalid".to_string(),
            amount: Uint128::new(MIN_MINT_PRICE_OPEN_EDITION),
        }),
        None,
        None,
        None,
    );
    assert_eq!(
        vt.err()
            .unwrap()
            .err()
            .unwrap()
            .source()
            .unwrap()
            .to_string(),
        "InvalidDenom".to_string()
    );

    // Invalid price
    let vt = open_edition_minter_custom_template(
        None,
        None,
        None,
        Some(10),
        Some(2),
        Some(Coin {
            denom: NATIVE_DENOM.to_string(),
            amount: Uint128::new(100u128),
        }),
        None,
        None,
        None,
    );
    assert_eq!(
        vt.err()
            .unwrap()
            .err()
            .unwrap()
            .source()
            .unwrap()
            .to_string(),
        "InvalidMintPrice".to_string()
    );
}

#[test]
fn check_custom_create_minter_denom() {
    // allow ibc/frenz denom
    let vt = open_edition_minter_custom_template(
        None,
        None,
        None,
        Some(10),
        Some(2),
        Some(Coin {
            denom: "uinvalid".to_string(),
            amount: Uint128::new(MIN_MINT_PRICE_OPEN_EDITION),
        }),
        None,
        None,
        None,
    );
    assert_eq!(
        vt.err()
            .unwrap()
            .err()
            .unwrap()
            .source()
            .unwrap()
            .to_string(),
        "InvalidDenom".to_string()
    );
}

#[test]
fn check_invalid_create_minter_nft_data() {
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
    let token_uri_def = Some(
        "ipfs://bafybeigi3bwpvyvsmnbj46ra4hyffcxdeaj6ntfk5jpic5mx27x6ih2qvq/images/1.png"
            .to_string(),
    );

    // Sending None for extension and token_uri
    let vt = open_edition_minter_custom_template(
        None,
        None,
        Some(NftData {
            nft_data_type: NftMetadataType::OffChainMetadata,
            extension: None,
            token_uri: None,
        }),
        None,
        None,
        None,
        None,
        None,
        None,
    );
    assert_eq!(
        vt.err()
            .unwrap()
            .err()
            .unwrap()
            .source()
            .unwrap()
            .to_string(),
        "InvalidNftDataProvided".to_string()
    );

    // Sending None for token_uri but offchain metadata
    let vt = open_edition_minter_custom_template(
        None,
        None,
        Some(NftData {
            nft_data_type: NftMetadataType::OffChainMetadata,
            extension: metadata_def,
            token_uri: None,
        }),
        None,
        None,
        None,
        None,
        None,
        None,
    );
    assert_eq!(
        vt.err()
            .unwrap()
            .err()
            .unwrap()
            .source()
            .unwrap()
            .to_string(),
        "InvalidNftDataProvided".to_string()
    );

    // Sending None for extension but onchain metadata
    let vt = open_edition_minter_custom_template(
        None,
        None,
        Some(NftData {
            nft_data_type: NftMetadataType::OnChainMetadata,
            extension: None,
            token_uri: token_uri_def.clone(),
        }),
        None,
        None,
        None,
        None,
        None,
        None,
    );
    assert_eq!(
        vt.err()
            .unwrap()
            .err()
            .unwrap()
            .source()
            .unwrap()
            .to_string(),
        "InvalidNftDataProvided".to_string()
    );

    // Sending extension and token_uri
    let vt = open_edition_minter_custom_template(
        None,
        None,
        Some(NftData {
            nft_data_type: NftMetadataType::OnChainMetadata,
            extension: None,
            token_uri: token_uri_def,
        }),
        None,
        None,
        None,
        None,
        None,
        None,
    );
    assert_eq!(
        vt.err()
            .unwrap()
            .err()
            .unwrap()
            .source()
            .unwrap()
            .to_string(),
        "InvalidNftDataProvided".to_string()
    );
}
