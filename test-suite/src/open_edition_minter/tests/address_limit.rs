use cosmwasm_std::coins;
use cw_multi_test::Executor;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};

use open_edition_minter::msg::ConfigResponse;
use open_edition_minter::msg::{ExecuteMsg, QueryMsg};

use crate::common_setup::setup_accounts_and_block::setup_block_time;
use crate::common_setup::templates::{open_edition_minter_custom_template, DEFAULT_CUSTOM_PARAMS};

const MINT_PRICE: u128 = 100_000_000;

#[test]
fn check_per_address_limit() {
    let vt = open_edition_minter_custom_template(
        None,
        None,
        None,
        Some(10),
        Some(2),
        None,
        DEFAULT_CUSTOM_PARAMS,
        None,
        None,
    )
    .unwrap();
    let (mut router, creator, buyer) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();
    // Set to a valid mint time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 101, None);

    // Check the Config
    let query_config_msg = QueryMsg::Config {};
    let res: ConfigResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &query_config_msg)
        .unwrap();
    assert_eq!(res.per_address_limit, 2);

    // Set a new limit per address, check unauthorized
    let per_address_limit_msg = ExecuteMsg::UpdatePerAddressLimit {
        per_address_limit: 5,
    };
    let res = router.execute_contract(
        buyer.clone(), // unauthorized
        minter_addr.clone(),
        &per_address_limit_msg,
        &[],
    );
    assert_eq!(
        res.err().unwrap().source().unwrap().to_string(),
        "Unauthorized: Sender is not an admin"
    );

    // Set limit errors, invalid limit over max
    // Factory is set to 10 in the current case
    let per_address_limit_msg = ExecuteMsg::UpdatePerAddressLimit {
        per_address_limit: 30,
    };
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &per_address_limit_msg,
        &[],
    );
    assert_eq!(
        res.err().unwrap().source().unwrap().to_string(),
        "Invalid minting limit per address. max: 10, min: 1, got: 30"
    );

    // Set limit errors, invalid limit == 0
    let per_address_limit_msg = ExecuteMsg::UpdatePerAddressLimit {
        per_address_limit: 0,
    };
    let res = router.execute_contract(creator, minter_addr.clone(), &per_address_limit_msg, &[]);
    assert_eq!(
        res.err().unwrap().source().unwrap().to_string(),
        "Invalid minting limit per address. max: 10, min: 1, got: 0"
    );

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

    // 3rd mint fails from exceeding per address limit
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer,
        minter_addr,
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert_eq!(
        res.err().unwrap().source().unwrap().to_string(),
        "Max minting limit per address exceeded"
    );
}
