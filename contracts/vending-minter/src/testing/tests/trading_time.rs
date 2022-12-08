use crate::msg::{ExecuteMsg, QueryMsg, StartTimeResponse};
use crate::ContractError;
use cosmwasm_std::{coin, coins, Addr, Timestamp};
use cw_multi_test::Executor;
use sg2::msg::Sg2ExecuteMsg;
use sg721_base::msg::{CollectionInfoResponse, QueryMsg as Sg721QueryMsg};
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};

use crate::testing::setup::setup_contracts::{
    contract_factory, contract_minter, contract_sg721, custom_mock_app, mock_create_minter,
    mock_params, setup_minter_contract,
};

use crate::testing::setup::setup_accounts_and_block::{setup_accounts, setup_block_time};
const CREATION_FEE: u128 = 5_000_000_000;

const MINT_PRICE: u128 = 100_000_000;

#[test]
fn before_start_time() {
    let mut router = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut router);
    let num_tokens = 1;
    let (minter_addr, _) = setup_minter_contract(&mut router, &creator, num_tokens, None);

    // Set to before genesis mint start time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME - 10, None);

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
    let mut router = custom_mock_app();
    let (creator, _) = setup_accounts(&mut router);
    let num_tokens = 10;

    let (minter_addr, _) = setup_minter_contract(&mut router, &creator, num_tokens, None);

    // Public mint has started
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 100, None);

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

    // Upload contract code
    let sg721_code_id = router.store_code(contract_sg721());
    let minter_code_id = router.store_code(contract_minter());
    let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);

    let factory_code_id = router.store_code(contract_factory());

    let mut params = mock_params();
    params.code_id = minter_code_id;

    let factory_addr = router
        .instantiate_contract(
            factory_code_id,
            creator.clone(),
            &vending_factory::msg::InstantiateMsg { params },
            &[],
            "factory",
            None,
        )
        .unwrap();

    // set time before the start_time above
    setup_block_time(&mut router, GENESIS_MINT_START_TIME - 1000, None);

    // Instantiate sale contract before genesis mint
    // let mut minter_init_msg = minter_init();
    let mut minter_msg = mock_create_minter(None);
    minter_msg.init_msg.num_tokens = 10;
    minter_msg.collection_params.code_id = sg721_code_id;
    minter_msg.init_msg.start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME - 100);
    let msg = Sg2ExecuteMsg::CreateMinter(minter_msg.clone());

    router
        .execute_contract(creator.clone(), factory_addr.clone(), &msg, &creation_fee)
        .unwrap_err();

    // move date after genesis mint
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 1000, None);

    // move start time after genesis but before current time
    minter_msg.init_msg.start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 500);
    let msg = Sg2ExecuteMsg::CreateMinter(minter_msg.clone());
    router
        .execute_contract(creator.clone(), factory_addr.clone(), &msg, &creation_fee)
        .unwrap_err();

    // position block time before the start time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 400, None);
    minter_msg.init_msg.start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 500);
    let msg = Sg2ExecuteMsg::CreateMinter(minter_msg);
    router
        .execute_contract(creator.clone(), factory_addr, &msg, &creation_fee)
        .unwrap();

    let minter_addr = Addr::unchecked("contract1");

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

    let minter_code_id = router.store_code(contract_minter());
    println!("minter_code_id: {}", minter_code_id);
    let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);

    let factory_code_id = router.store_code(contract_factory());
    println!("factory_code_id: {}", factory_code_id);

    // set up minter contract
    let mut params = mock_params();
    params.code_id = minter_code_id;
    let factory_addr = router
        .instantiate_contract(
            factory_code_id,
            creator.clone(),
            &vending_factory::msg::InstantiateMsg {
                params: params.clone(),
            },
            &[],
            "factory",
            None,
        )
        .unwrap();

    let sg721_code_id = router.store_code(contract_sg721());
    println!("sg721_code_id: {}", sg721_code_id);

    let mut msg = mock_create_minter(None);
    msg.init_msg.mint_price = coin(MINT_PRICE, NATIVE_DENOM);
    msg.init_msg.num_tokens = num_tokens;
    msg.collection_params.code_id = sg721_code_id;
    msg.collection_params.info.creator = creator.to_string();
    // make trading time beyond factory max trading start time offset
    msg.collection_params.info.start_trading_time = Some(
        msg.init_msg
            .start_time
            .plus_seconds(params.max_trading_offset_secs + 1),
    );

    let msg = Sg2ExecuteMsg::CreateMinter(msg);

    let err = router
        .execute_contract(creator, factory_addr, &msg, &creation_fee)
        .unwrap_err();
    assert!(err
        .source()
        .unwrap()
        .source()
        .unwrap()
        .to_string()
        .contains("InvalidStartTradingTime"));
}

#[test]
fn update_start_trading_time() {
    let mut router = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut router);
    let num_tokens = 2;
    setup_block_time(&mut router, GENESIS_MINT_START_TIME - 1, None);
    let (minter_addr, config) = setup_minter_contract(&mut router, &creator, num_tokens, None);

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
    let params = mock_params();
    let res = router.execute_contract(
        Addr::unchecked(creator.clone()),
        Addr::unchecked(minter_addr.clone()),
        &ExecuteMsg::UpdateStartTradingTime(Some(
            Timestamp::from_nanos(GENESIS_MINT_START_TIME)
                .plus_seconds(params.max_trading_offset_secs + 100),
        )),
        &[],
    );
    assert!(res.is_err());

    // succeeds
    let res = router.execute_contract(
        Addr::unchecked(creator.clone()),
        Addr::unchecked(minter_addr),
        &ExecuteMsg::UpdateStartTradingTime(Some(
            Timestamp::from_nanos(GENESIS_MINT_START_TIME)
                .plus_seconds(params.max_trading_offset_secs),
        )),
        &[],
    );
    assert!(res.is_ok());

    // confirm trading start time
    let res: CollectionInfoResponse = router
        .wrap()
        .query_wasm_smart(config.sg721_address, &Sg721QueryMsg::CollectionInfo {})
        .unwrap();
    assert_eq!(
        res.start_trading_time,
        Some(
            Timestamp::from_nanos(GENESIS_MINT_START_TIME)
                .plus_seconds(params.max_trading_offset_secs)
        )
    );
}
