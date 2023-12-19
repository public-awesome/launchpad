use cosmwasm_std::{Coin, Uint128};
use open_edition_factory::state::ParamsExtension;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};

use open_edition_minter::msg::ConfigResponse;
use open_edition_minter::msg::QueryMsg;

use crate::common_setup::setup_accounts_and_block::setup_block_time;
use crate::common_setup::setup_minter::common::constants::DEV_ADDRESS;
use crate::common_setup::setup_minter::open_edition_minter::minter_params::{
    init_msg, nft_data_with_uri_scheme,
};
use crate::common_setup::templates::open_edition_minter_custom_uri_scheme;

const MINT_PRICE: u128 = 100_000_000;

#[test]
fn check_expected_uri_scheme_in_minter_config() {
    let params_extension = ParamsExtension {
        max_per_address_limit: 10,
        airdrop_mint_fee_bps: 100,
        airdrop_mint_price: Coin {
            denom: NATIVE_DENOM.to_string(),
            amount: Uint128::new(MINT_PRICE),
        },
        dev_fee_address: DEV_ADDRESS.to_string(),
    };

    let per_address_limit_minter = Some(2);
    let exp_uri_scheme = "https".to_owned();

    let init_msg = init_msg(
        nft_data_with_uri_scheme(exp_uri_scheme.clone()),
        per_address_limit_minter,
        None,
        None,
        None,
    );

    let vt = open_edition_minter_custom_uri_scheme(
        params_extension,
        init_msg,
        Some(exp_uri_scheme.clone()),
    )
    .unwrap();

    let (mut router, ..) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();

    // Set to a valid mint time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 101, None);

    // Check the Config
    let query_config_msg = QueryMsg::Config {};
    let res: ConfigResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &query_config_msg)
        .unwrap();

    assert_eq!(res.uri_scheme, exp_uri_scheme);
}

#[test]
fn check_error_on_unexpected_token_uri_scheme() {
    let params_extension = ParamsExtension {
        max_per_address_limit: 10,
        airdrop_mint_fee_bps: 100,
        airdrop_mint_price: Coin {
            denom: NATIVE_DENOM.to_string(),
            amount: Uint128::new(MINT_PRICE),
        },
        dev_fee_address: DEV_ADDRESS.to_string(),
    };

    let per_address_limit_minter = Some(2);
    let exp_uri_scheme = "https".to_owned();
    let invalid_uri_scheme = "ipfs".to_owned();

    let init_msg = init_msg(
        nft_data_with_uri_scheme(invalid_uri_scheme),
        per_address_limit_minter,
        None,
        None,
        None,
    );

    let vt = open_edition_minter_custom_uri_scheme(
        params_extension,
        init_msg,
        Some(exp_uri_scheme.clone()),
    )
    .unwrap();

    let (_router, _creator, _buyer) = (vt.router, vt.accts.creator, vt.accts.buyer);

    let error = vt.collection_response_vec[0].error.as_ref();
    let error_str = format!("{:?}", error);

    assert!(error.is_some());
    assert!(error_str.contains(&format!("must be an {} URI", exp_uri_scheme)));
}
