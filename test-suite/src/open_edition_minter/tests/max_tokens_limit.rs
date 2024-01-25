use cosmwasm_std::{coins, Coin, Timestamp, Uint128};
use cw_multi_test::Executor;
use open_edition_factory::state::ParamsExtension;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};

use open_edition_minter::msg::ConfigResponse;
use open_edition_minter::msg::{ExecuteMsg, QueryMsg};

use crate::common_setup::setup_accounts_and_block::setup_block_time;
use crate::common_setup::setup_minter::common::constants::{DEV_ADDRESS, MAX_TOKEN_LIMIT};
use crate::common_setup::setup_minter::open_edition_minter::minter_params::{
    default_nft_data, init_msg,
};
use crate::common_setup::templates::open_edition_minter_custom_template;

const MINT_PRICE: u128 = 100_000_000;

#[test]
fn check_max_tokens_limit_init() {
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
    // if the number of tokens to be minted exceed to max, should error
    let per_address_limit_minter = Some(2);
    let init_msg = init_msg(
        default_nft_data(),
        per_address_limit_minter,
        None,
        Some(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10_000)),
        Some(MAX_TOKEN_LIMIT + 1),
        None,
    );
    let vt = open_edition_minter_custom_template(params_extension.clone(), init_msg).unwrap();
    assert!(vt.collection_response_vec[0].error.is_some());
    // Should work otherwise
    let init_msg = crate::common_setup::setup_minter::open_edition_minter::minter_params::init_msg(
        default_nft_data(),
        per_address_limit_minter,
        None,
        Some(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10_000)),
        Some(2),
        None,
    );
    let vt = open_edition_minter_custom_template(params_extension, init_msg);
    assert!(vt.is_ok());

    let response = vt.unwrap();
    let (mut router, creator, buyer) = (
        response.router,
        response.accts.creator,
        response.accts.buyer,
    );
    let minter_addr = response.collection_response_vec[0].minter.clone().unwrap();
    // Set to a valid mint time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 101, None);

    // Check the Config for the num_tokens value
    let query_config_msg = QueryMsg::Config {};
    let res: ConfigResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &query_config_msg)
        .unwrap();
    assert_eq!(res.num_tokens, Some(2));

    // Only the first 2 mints
    for _ in 1..=2 {
        let mint_msg = ExecuteMsg::Mint {};
        let res = router.execute_contract(
            buyer.clone(),
            minter_addr.clone(),
            &mint_msg,
            &coins(MINT_PRICE, NATIVE_DENOM),
        );
        assert!(res.is_ok());
    }
    // 3rd mint fails from exceeding num of tokens
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        creator,
        minter_addr,
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert_eq!(res.err().unwrap().source().unwrap().to_string(), "Sold out");
}
