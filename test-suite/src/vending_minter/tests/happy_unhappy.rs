use crate::common_setup::setup_minter::vending_minter::mock_params::mock_create_minter;
use crate::common_setup::templates::vending_minter_template;
use crate::common_setup::{
    setup_accounts_and_block::coins_for_msg, setup_accounts_and_block::setup_block_time,
};
use cosmwasm_std::{
    coin, coins,
    testing::{mock_dependencies_with_balance, mock_env, mock_info},
    Api, Coin, Timestamp, Uint128,
};
use cw721::{Cw721QueryMsg, OwnerOfResponse};
use cw_multi_test::Executor;
use sg2::tests::mock_collection_params_1;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
use vending_minter::msg::{ExecuteMsg, QueryMsg, StartTimeResponse};
use vending_minter::{contract::instantiate, msg::MintCountResponse};

const INITIAL_BALANCE: u128 = 2_000_000_000;
const MINT_PRICE: u128 = 100_000_000;
const MINT_FEE: u128 = 10_000_000;
const ADMIN_MINT_PRICE: u128 = 0;
const MAX_TOKEN_LIMIT: u32 = 10000;

#[test]
fn initialization() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    // Check valid addr
    let addr = "earth1";
    let res = deps.api.addr_validate(addr);
    assert!(res.is_ok());

    // 0 per address limit returns error
    let info = mock_info("creator", &coins(INITIAL_BALANCE, NATIVE_DENOM));
    // let mut msg = minter_init();

    let start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    let collection_params = mock_collection_params_1(Some(start_time));
    let mut msg = mock_create_minter(None, collection_params.clone(), None);
    msg.init_msg.num_tokens = 100;
    msg.collection_params.code_id = 1;
    msg.collection_params.info.creator = info.sender.to_string();

    instantiate(deps.as_mut(), mock_env(), info, msg.clone()).unwrap_err();

    // Invalid uri returns error
    let info = mock_info("creator", &coins(INITIAL_BALANCE, NATIVE_DENOM));
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    // Invalid denom returns error
    let wrong_denom = "uosmo";
    let info = mock_info("creator", &coins(INITIAL_BALANCE, NATIVE_DENOM));
    // let mut msg = minter_init();
    let mut msg = mock_create_minter(None, collection_params.clone(), None);
    // msg.init_msg.mint_price = 100;
    msg.init_msg.mint_price = coin(MINT_PRICE, wrong_denom);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    // Insufficient mint price returns error
    let info = mock_info("creator", &coins(INITIAL_BALANCE, NATIVE_DENOM));
    let mut msg = mock_create_minter(None, collection_params.clone(), None);
    msg.init_msg.mint_price = coin(1, NATIVE_DENOM);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    // Over max token limit
    let info = mock_info("creator", &coins(INITIAL_BALANCE, NATIVE_DENOM));
    // let mut msg = minter_init();
    let mut msg = mock_create_minter(None, collection_params.clone(), None);
    msg.init_msg.mint_price = coin(MINT_PRICE, NATIVE_DENOM);
    msg.init_msg.num_tokens = MAX_TOKEN_LIMIT + 1;

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    // Under min token limit
    let info = mock_info("creator", &coins(INITIAL_BALANCE, NATIVE_DENOM));
    // let mut msg = minter_init();
    let mut msg = mock_create_minter(None, collection_params, None);
    msg.init_msg.num_tokens = 0;

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();
}

#[test]
fn happy_path() {
    let vt = vending_minter_template(2);
    let (mut router, creator, buyer) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();
    let collection_addr = vt.collection_response_vec[0].collection.clone().unwrap();

    // Default start time genesis mint time
    let res: StartTimeResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &QueryMsg::StartTime {})
        .unwrap();
    assert_eq!(
        res.start_time,
        Timestamp::from_nanos(GENESIS_MINT_START_TIME).to_string()
    );

    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 1, None);

    // Fail with incorrect tokens
    let mint_msg = ExecuteMsg::Mint {};
    let err = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(MINT_PRICE + 100, NATIVE_DENOM),
    );
    assert!(err.is_err());

    // Succeeds if funds are sent
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_ok());

    // Balances are correct
    // The creator should get the unit price - mint fee for the mint above
    let creator_balances = router.wrap().query_all_balances(creator.clone()).unwrap();
    assert_eq!(
        creator_balances,
        coins(INITIAL_BALANCE + MINT_PRICE - MINT_FEE, NATIVE_DENOM)
    );
    // The buyer's tokens should reduce by unit price
    let buyer_balances = router.wrap().query_all_balances(buyer.clone()).unwrap();
    assert_eq!(
        buyer_balances,
        coins(INITIAL_BALANCE - MINT_PRICE, NATIVE_DENOM)
    );

    let res: MintCountResponse = router
        .wrap()
        .query_wasm_smart(
            minter_addr.clone(),
            &QueryMsg::MintCount {
                address: buyer.to_string(),
            },
        )
        .unwrap();
    assert_eq!(res.count, 1);
    assert_eq!(res.address, buyer.to_string());

    // Check NFT owned by buyer
    // Random mint token_id 1
    let query_owner_msg = Cw721QueryMsg::OwnerOf {
        token_id: String::from("2"),
        include_expired: None,
    };

    let res: OwnerOfResponse = router
        .wrap()
        .query_wasm_smart(collection_addr.clone(), &query_owner_msg)
        .unwrap();
    assert_eq!(res.owner, buyer.to_string());

    // Buyer can't call MintTo
    let mint_to_msg = ExecuteMsg::MintTo {
        recipient: buyer.to_string(),
    };
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_to_msg,
        &coins_for_msg(Coin {
            amount: Uint128::from(ADMIN_MINT_PRICE),
            denom: NATIVE_DENOM.to_string(),
        }),
    );
    assert!(res.is_err());

    // Creator mints an extra NFT for the buyer (who is a friend)
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &mint_to_msg,
        &coins_for_msg(Coin {
            amount: Uint128::from(ADMIN_MINT_PRICE),
            denom: NATIVE_DENOM.to_string(),
        }),
    );
    assert!(res.is_ok());

    // Mint count is not increased if admin mints for the user
    let res: MintCountResponse = router
        .wrap()
        .query_wasm_smart(
            minter_addr.clone(),
            &QueryMsg::MintCount {
                address: buyer.to_string(),
            },
        )
        .unwrap();
    assert_eq!(res.count, 1);
    assert_eq!(res.address, buyer.to_string());

    // Minter contract should have no balance
    let minter_balance = router
        .wrap()
        .query_all_balances(minter_addr.clone())
        .unwrap();
    assert_eq!(0, minter_balance.len());

    // Check that NFT is transferred
    let query_owner_msg = Cw721QueryMsg::OwnerOf {
        token_id: String::from("1"),
        include_expired: None,
    };
    let res: OwnerOfResponse = router
        .wrap()
        .query_wasm_smart(collection_addr, &query_owner_msg)
        .unwrap();
    assert_eq!(res.owner, buyer.to_string());

    // Errors if sold out
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins_for_msg(Coin {
            amount: Uint128::from(MINT_PRICE),
            denom: NATIVE_DENOM.to_string(),
        }),
    );
    assert!(res.is_err());

    // Creator can't use MintTo if sold out
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &mint_to_msg,
        &coins_for_msg(Coin {
            amount: Uint128::from(ADMIN_MINT_PRICE),
            denom: NATIVE_DENOM.to_string(),
        }),
    );
    assert!(res.is_err());

    // Can purge after sold out
    let purge_msg = ExecuteMsg::Purge {};
    let res = router.execute_contract(creator, minter_addr.clone(), &purge_msg, &[]);
    assert!(res.is_ok());

    // MintCount should be 0 after purge
    let res: MintCountResponse = router
        .wrap()
        .query_wasm_smart(
            minter_addr,
            &QueryMsg::MintCount {
                address: buyer.to_string(),
            },
        )
        .unwrap();
    assert_eq!(res.count, 0);
}

#[test]
fn unhappy_path() {
    let vt = vending_minter_template(2);
    let (mut router, _, buyer) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();
    // Fails if too little funds are sent
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(1, NATIVE_DENOM),
    );
    assert!(res.is_err());

    // Fails if too many funds are sent
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(11111, NATIVE_DENOM),
    );
    assert!(res.is_err());

    // Fails wrong denom is sent
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(buyer, minter_addr, &mint_msg, &coins(MINT_PRICE, "uatom"));
    assert!(res.is_err());
}
