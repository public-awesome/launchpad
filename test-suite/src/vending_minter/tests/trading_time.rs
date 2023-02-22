use crate::common_setup::contract_boxes::custom_mock_app;
use crate::common_setup::msg::{MinterCollectionResponse, MinterInstantiateParams};
use crate::common_setup::setup_accounts_and_block::{setup_accounts, setup_block_time};
use crate::common_setup::setup_minter::vending_minter::setup::{
    configure_minter, vending_minter_code_ids,
};
use crate::common_setup::templates::{vending_minter_with_app, vending_minter_with_start_time};
use cosmwasm_std::{coins, Addr, Timestamp};
use cw_multi_test::Executor;
use sg2::tests::mock_collection_params_1;
use sg721_base::msg::{CollectionInfoResponse, QueryMsg as Sg721QueryMsg};
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
use vending_minter::msg::{ExecuteMsg, QueryMsg, StartTimeResponse};
use vending_minter::ContractError;

const MINT_PRICE: u128 = 100_000_000;

#[test]
fn before_start_time() {
    let vt = vending_minter_with_start_time(1, Timestamp::from_nanos(GENESIS_MINT_START_TIME - 10));
    let (mut router, _, buyer) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();

    // Set start_time fails if not admin
    let start_time_msg = ExecuteMsg::UpdateStartTime(Timestamp::from_nanos(0));
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &start_time_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_err());

    // Buyer can't mint before start_time
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_err());

    // Query start_time, confirm expired
    let start_time_response: StartTimeResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &QueryMsg::StartTime {})
        .unwrap();
    assert_eq!(
        Timestamp::from_nanos(GENESIS_MINT_START_TIME).to_string(),
        start_time_response.start_time
    );

    // Set block forward, after start time. mint succeeds
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 10_000_000, None);

    // Mint succeeds
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer,
        minter_addr,
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_ok());
}

#[test]
fn test_update_start_time() {
    let start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 100);
    let vt = vending_minter_with_start_time(1, start_time);
    let (mut router, creator) = (vt.router, vt.accts.creator);
    let minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();

    setup_block_time(&mut router, start_time.nanos(), None);

    // Update to a start time in the past
    let msg = ExecuteMsg::UpdateStartTime(Timestamp::from_nanos(GENESIS_MINT_START_TIME - 1000));
    let err = router
        .execute_contract(creator, minter_addr, &msg, &[])
        .unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        ContractError::AlreadyStarted {}.to_string(),
    );
}

#[test]
fn test_invalid_start_time() {
    let mut router = custom_mock_app();
    let (creator, _) = setup_accounts(&mut router);
    let num_tokens = 10;
    setup_block_time(&mut router, GENESIS_MINT_START_TIME - 1000, None);
    let collection_start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    let collection_params = mock_collection_params_1(Some(collection_start_time));
    let minter_params = MinterInstantiateParams {
        num_tokens,
        splits_addr: None,
        start_time: Some(Timestamp::from_nanos(GENESIS_MINT_START_TIME - 100)),
        init_msg: None,
    };
    let code_ids = vending_minter_code_ids(&mut router);
    let minter_collection_response: Vec<MinterCollectionResponse> = configure_minter(
        &mut router,
        creator.clone(),
        vec![collection_params.clone()],
        vec![minter_params],
        code_ids,
    );

    let err = minter_collection_response[0]
        .error
        .as_ref()
        .unwrap()
        .root_cause();
    let expected_error = ContractError::BeforeGenesisTime {};
    assert_eq!(err.to_string(), expected_error.to_string());
    // set time before the start_time above

    // move date after genesis mint
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 1000, None);
    let minter_start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 500);
    let minter_params = MinterInstantiateParams {
        num_tokens,
        splits_addr: None,
        start_time: Some(minter_start_time),
        init_msg: None,
    };
    let code_ids = vending_minter_code_ids(&mut router);
    let minter_collection_response: Vec<MinterCollectionResponse> = configure_minter(
        &mut router,
        creator.clone(),
        vec![collection_params.clone()],
        vec![minter_params],
        code_ids.clone(),
    );

    let err = minter_collection_response[0]
        .error
        .as_ref()
        .unwrap()
        .root_cause();
    let expected_error =
        ContractError::InvalidStartTime(minter_start_time, router.block_info().time);
    assert_eq!(err.to_string(), expected_error.to_string());

    // position block time before the start time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 400, None);
    let minter_start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 500);
    let minter_params = MinterInstantiateParams {
        num_tokens,
        splits_addr: None,
        start_time: Some(minter_start_time),
        init_msg: None,
    };
    let minter_collection_response: Vec<MinterCollectionResponse> = configure_minter(
        &mut router,
        creator.clone(),
        vec![collection_params],
        vec![minter_params],
        code_ids,
    );
    let minter_addr = minter_collection_response[0].minter.clone().unwrap();
    assert_eq!(minter_addr.to_string(), "contract3");

    // Update to a start time in the past
    let msg = ExecuteMsg::UpdateStartTime(Timestamp::from_nanos(GENESIS_MINT_START_TIME - 100));
    let res = router.execute_contract(creator.clone(), minter_addr.clone(), &msg, &[]);
    assert!(res.is_err());

    // Update to a time after genesis but before the current block_time (GENESIS+400)
    let msg = ExecuteMsg::UpdateStartTime(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 300));
    let res = router.execute_contract(creator.clone(), minter_addr.clone(), &msg, &[]);
    assert!(res.is_err());

    // Update to a time after genesis and after current blocktime (GENESIS+400)
    let msg = ExecuteMsg::UpdateStartTime(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 450));
    let res = router.execute_contract(creator.clone(), minter_addr.clone(), &msg, &[]);
    assert!(res.is_ok());

    // position block after start time (GENESIS+450);
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 500, None);

    // Update to a time after genesis and after current blocktime (GENESIS+400)
    let msg = ExecuteMsg::UpdateStartTime(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 450));
    let err = router
        .execute_contract(creator, minter_addr, &msg, &[])
        .unwrap_err();
    assert_eq!(err.source().unwrap().to_string(), "AlreadyStarted");
}

#[test]
fn invalid_trading_time_during_init() {
    let num_tokens = 10;
    let mut router = custom_mock_app();
    let (creator, _) = setup_accounts(&mut router);

    // let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);
    setup_block_time(&mut router, GENESIS_MINT_START_TIME, None);
    let genesis_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    let max_trading_offset = 60 * 60 * 24 * 7;
    let collection_params =
        mock_collection_params_1(Some(genesis_time.plus_seconds(max_trading_offset + 1)));

    let minter_params = MinterInstantiateParams {
        num_tokens,
        splits_addr: None,
        start_time: None,
        init_msg: None,
    };
    let code_ids = vending_minter_code_ids(&mut router);
    let minter_collection_response: Vec<MinterCollectionResponse> = configure_minter(
        &mut router,
        creator,
        vec![collection_params],
        vec![minter_params],
        code_ids,
    );
    let err = minter_collection_response[0].error.as_deref();
    let expected_error = ContractError::InvalidStartTradingTime(
        genesis_time.plus_seconds(max_trading_offset + 1),
        genesis_time.plus_seconds(max_trading_offset),
    );
    assert_eq!(
        err.unwrap().source().unwrap().source().unwrap().to_string(),
        expected_error.to_string()
    );
}
#[test]
fn update_start_trading_time() {
    let mut router = custom_mock_app();
    setup_block_time(&mut router, GENESIS_MINT_START_TIME - 1, None);
    let vt = vending_minter_with_app(1, router);
    let (mut router, creator, buyer) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();
    let collection_addr = vt.collection_response_vec[0].collection.clone().unwrap();

    let max_trading_offset = 60 * 60 * 24 * 7;
    // unauthorized
    let res = router.execute_contract(
        Addr::unchecked(buyer),
        Addr::unchecked(minter_addr.clone()),
        &ExecuteMsg::UpdateStartTradingTime(Some(Timestamp::from_nanos(GENESIS_MINT_START_TIME))),
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

    // invalid start trading time, over offset
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &ExecuteMsg::UpdateStartTradingTime(Some(
            Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(max_trading_offset + 100),
        )),
        &[],
    );
    assert!(res.is_err());

    // succeeds
    let res = router.execute_contract(
        creator,
        minter_addr,
        &ExecuteMsg::UpdateStartTradingTime(Some(
            Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(max_trading_offset),
        )),
        &[],
    );
    assert!(res.is_ok());

    // confirm trading start time
    let res: CollectionInfoResponse = router
        .wrap()
        .query_wasm_smart(
            collection_addr.to_string(),
            &Sg721QueryMsg::CollectionInfo {},
        )
        .unwrap();

    assert_eq!(
        res.start_trading_time,
        Some(Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(max_trading_offset))
    );
}
