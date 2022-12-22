use crate::common_setup::setup_accounts_and_block::{CREATION_FEE, INITIAL_BALANCE};
use crate::common_setup::setup_minter::base_minter::mock_params::mock_params;
use crate::common_setup::setup_minter::common::constants::MIN_MINT_PRICE;
use crate::common_setup::templates::{base_minter_with_sg721, base_minter_with_sg721nt};
use base_minter::msg::{ConfigResponse, ExecuteMsg};
use cosmwasm_std::{coin, coins, Addr, Timestamp};
use cw721::{Cw721ExecuteMsg, Cw721QueryMsg, OwnerOfResponse};
use cw_multi_test::Executor;
use sg4::QueryMsg;
use sg721_base::msg::{CollectionInfoResponse, QueryMsg as Sg721QueryMsg};
use sg_std::NATIVE_DENOM;

#[test]
fn check_mint() {
    let bmt = base_minter_with_sg721nt(1);
    let (mut router, creator, buyer) = (bmt.router, bmt.creator, bmt.buyer);
    let minter_addr = bmt.collection_response_vec[0].minter.clone().unwrap();
    let collection_addr = bmt.collection_response_vec[0].collection.clone().unwrap();

    // Fail with incorrect token uri
    let mint_msg = ExecuteMsg::Mint {
        token_uri: "test uri".to_string(),
    };
    let err = router.execute_contract(creator.clone(), minter_addr.clone(), &mint_msg, &[]);
    assert!(err.is_err());

    // Fail with incorrect mint price
    let mint_msg = ExecuteMsg::Mint {
        token_uri: "ipfs://example".to_string(),
    };
    let err = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &mint_msg,
        &[coin(MIN_MINT_PRICE + 100, NATIVE_DENOM)],
    );
    assert!(err.is_err());

    // Not authorized to mint
    let mint_msg = ExecuteMsg::Mint {
        token_uri: "ipfs://example".to_string(),
    };
    let err = router.execute_contract(
        buyer,
        minter_addr.clone(),
        &mint_msg,
        &[coin(MIN_MINT_PRICE, NATIVE_DENOM)],
    );
    assert!(err.is_err());

    // Succeeds if funds are sent
    let mint_msg = ExecuteMsg::Mint {
        token_uri: "ipfs://example".to_string(),
    };
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &mint_msg,
        &[coin(MIN_MINT_PRICE, NATIVE_DENOM)],
    );

    println!("res is {:?}", res);
    assert!(res.is_ok());

    let creator_balances = router.wrap().query_all_balances(creator.clone()).unwrap();
    assert_eq!(
        creator_balances,
        coins(
            (INITIAL_BALANCE + CREATION_FEE) - CREATION_FEE - MIN_MINT_PRICE,
            NATIVE_DENOM
        )
    );

    let res: ConfigResponse = router
        .wrap()
        .query_wasm_smart(minter_addr, &QueryMsg::Config {})
        .unwrap();
    assert_eq!(res.collection_address, "contract2".to_string());
    assert_eq!(res.config.mint_price.amount.u128(), MIN_MINT_PRICE);

    let query_owner_msg = Cw721QueryMsg::OwnerOf {
        token_id: String::from("1"),
        include_expired: None,
    };
    let res: OwnerOfResponse = router
        .wrap()
        .query_wasm_smart(collection_addr.clone(), &query_owner_msg)
        .unwrap();
    assert_eq!(res.owner, creator.to_string());

    // make sure sg721-nt cannot be transferred
    let transfer_msg = Cw721ExecuteMsg::TransferNft {
        recipient: "adsf".to_string(),
        token_id: "1".to_string(),
    };
    let err = router.execute_contract(
        creator,
        Addr::unchecked(collection_addr),
        &transfer_msg,
        &[],
    );
    assert!(err.is_err());
}

#[test]
fn update_start_trading_time() {
    let bmt = base_minter_with_sg721(1);
    let (mut router, creator, buyer) = (bmt.router, bmt.creator, bmt.buyer);
    let minter_addr = bmt.collection_response_vec[0].minter.clone().unwrap();
    let collection_addr = bmt.collection_response_vec[0].collection.clone().unwrap();
    let current_block_time = router.block_info().time;

    let default_start_trading_time =
        current_block_time.plus_seconds(mock_params().max_trading_offset_secs + 1);

    // unauthorized
    let res = router.execute_contract(
        Addr::unchecked(buyer),
        Addr::unchecked(minter_addr.clone()),
        &ExecuteMsg::UpdateStartTradingTime(Some(default_start_trading_time)),
        &[],
    );
    assert!(res.is_err());

    // invalid start trading time
    let res = router.execute_contract(
        Addr::unchecked(creator.clone()),
        Addr::unchecked(minter_addr.clone()),
        &ExecuteMsg::UpdateStartTradingTime(Some(Timestamp::from_nanos(0))),
        &[],
    );
    assert!(res.is_err());

    // succeeds
    let res = router.execute_contract(
        Addr::unchecked(creator),
        Addr::unchecked(minter_addr),
        &ExecuteMsg::UpdateStartTradingTime(Some(default_start_trading_time)),
        &[],
    );
    println!("res is {:?}", res);
    assert!(res.is_ok());

    // confirm trading start time
    let res: CollectionInfoResponse = router
        .wrap()
        .query_wasm_smart(collection_addr, &Sg721QueryMsg::CollectionInfo {})
        .unwrap();
    assert_eq!(res.start_trading_time, Some(default_start_trading_time));
}
