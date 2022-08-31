use crate::msg::{ConfigResponse, ExecuteMsg};
use base_factory::msg::BaseMinterCreateMsg;
use base_factory::state::BaseMinterParams;
use cosmwasm_std::{coin, coins, Addr, Timestamp};
use cw721::{Cw721ExecuteMsg, Cw721QueryMsg, OwnerOfResponse};
use cw_multi_test::{BankSudo, Contract, ContractWrapper, Executor, SudoMsg};
use sg2::msg::Sg2ExecuteMsg;
use sg2::tests::mock_collection_params;
use sg4::QueryMsg;
use sg_multi_test::StargazeApp;
use sg_std::{StargazeMsgWrapper, NATIVE_DENOM};

const CREATION_FEE: u128 = 1_000_000_000;
const INITIAL_BALANCE: u128 = 2_000_000_000;
const MIN_MINT_PRICE: u128 = 50_000_000;
const MINT_FEE_BPS: u64 = 10_000; // 100%

fn custom_mock_app() -> StargazeApp {
    StargazeApp::default()
}

pub fn contract_factory() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        base_factory::contract::execute,
        base_factory::contract::instantiate,
        base_factory::contract::query,
    );
    Box::new(contract)
}

pub fn contract_minter() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    )
    .with_reply(crate::contract::reply);
    Box::new(contract)
}

/// non-transferable nft
pub fn contract_collection() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        sg721_nt::entry::execute,
        sg721_nt::entry::instantiate,
        sg721_nt::entry::query,
    );
    Box::new(contract)
}

/// standard sg721 nft
pub fn standard_contract_collection() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        sg721_base::entry::execute,
        sg721_base::entry::instantiate,
        sg721_base::entry::query,
    );
    Box::new(contract)
}

pub fn mock_params() -> BaseMinterParams {
    BaseMinterParams {
        code_id: 1,
        creation_fee: coin(CREATION_FEE, NATIVE_DENOM),
        min_mint_price: coin(MIN_MINT_PRICE, NATIVE_DENOM),
        mint_fee_bps: MINT_FEE_BPS,
        max_trading_start_time_offset: 5,
        extension: None,
    }
}

pub fn mock_create_minter() -> BaseMinterCreateMsg {
    BaseMinterCreateMsg {
        init_msg: None,
        collection_params: mock_collection_params(),
    }
}

// Upload contract code and instantiate minter contract
fn setup_minter_contract(router: &mut StargazeApp, creator: &Addr) -> (Addr, ConfigResponse) {
    let minter_code_id = router.store_code(contract_minter());
    let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);

    let factory_code_id = router.store_code(contract_factory());

    let mut params = mock_params();
    params.code_id = minter_code_id;

    let factory_addr = router
        .instantiate_contract(
            factory_code_id,
            creator.clone(),
            &base_factory::msg::InstantiateMsg { params },
            &[],
            "factory",
            None,
        )
        .unwrap();

    let collection_code_id = router.store_code(contract_collection());

    let mut msg = mock_create_minter();
    msg.collection_params.code_id = collection_code_id;
    msg.collection_params.info.creator = creator.to_string();

    let msg = Sg2ExecuteMsg::CreateMinter(msg);

    let balances = router.wrap().query_all_balances(creator.clone()).unwrap();
    assert_eq!(balances, coins(INITIAL_BALANCE, NATIVE_DENOM));

    let res = router.execute_contract(creator.clone(), factory_addr, &msg, &creation_fee);
    assert!(res.is_ok());

    let balances = router.wrap().query_all_balances(creator.clone()).unwrap();
    assert_eq!(
        balances,
        coins(INITIAL_BALANCE - CREATION_FEE, NATIVE_DENOM)
    );

    // could get the minter address from the response above, but we know its contract1
    let minter_addr = Addr::unchecked("contract1");

    let config: ConfigResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &QueryMsg::Config {})
        .unwrap();

    (minter_addr, config)
}

// Add a creator account with initial balances
fn setup_accounts(router: &mut StargazeApp) -> Addr {
    let creator = Addr::unchecked("creator");
    // 3,000 tokens
    let creator_funds = coins(INITIAL_BALANCE, NATIVE_DENOM);
    router
        .sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: creator.to_string(),
                amount: creator_funds.clone(),
            }
        }))
        .map_err(|err| println!("{:?}", err))
        .ok();

    let creator_native_balances = router.wrap().query_all_balances(creator.clone()).unwrap();
    assert_eq!(creator_native_balances, creator_funds);

    creator
}

#[test]
fn check_mint() {
    let mut router = custom_mock_app();
    let creator = setup_accounts(&mut router);
    let (minter_addr, config) = setup_minter_contract(&mut router, &creator);

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
    assert!(res.is_ok());

    let creator_balances = router.wrap().query_all_balances(creator.clone()).unwrap();
    assert_eq!(
        creator_balances,
        coins(
            INITIAL_BALANCE - CREATION_FEE - MIN_MINT_PRICE,
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
        .query_wasm_smart(config.collection_address.clone(), &query_owner_msg)
        .unwrap();
    assert_eq!(res.owner, creator.to_string());

    // make sure sg721-nt cannot be transferred
    let transfer_msg = Cw721ExecuteMsg::TransferNft {
        recipient: "adsf".to_string(),
        token_id: "1".to_string(),
    };
    let err = router.execute_contract(
        creator.clone(),
        Addr::unchecked(config.collection_address),
        &transfer_msg,
        &[],
    );
    assert!(err.is_err());
}

#[test]
fn start_trading_time() {
    let mut router = custom_mock_app();
    let creator = setup_accounts(&mut router);

    let minter_code_id = router.store_code(contract_minter());
    let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);

    let factory_code_id = router.store_code(contract_factory());

    let mut params = mock_params();
    params.code_id = minter_code_id;

    let factory_addr = router
        .instantiate_contract(
            factory_code_id,
            creator.clone(),
            &base_factory::msg::InstantiateMsg {
                params: params.clone(),
            },
            &[],
            "factory",
            None,
        )
        .unwrap();

    let collection_code_id = router.store_code(standard_contract_collection());

    let mut msg = mock_create_minter();
    msg.collection_params.code_id = collection_code_id;
    msg.collection_params.info.creator = creator.to_string();

    let msg = Sg2ExecuteMsg::CreateMinter(msg);

    let balances = router.wrap().query_all_balances(creator.clone()).unwrap();
    assert_eq!(balances, coins(INITIAL_BALANCE, NATIVE_DENOM));

    let res = router.execute_contract(creator.clone(), factory_addr, &msg, &creation_fee);
    assert!(res.is_ok());

    let balances = router.wrap().query_all_balances(creator.clone()).unwrap();
    assert_eq!(
        balances,
        coins(INITIAL_BALANCE - CREATION_FEE, NATIVE_DENOM)
    );

    // could get the minter address from the response above, but we know its contract1
    let minter_addr = Addr::unchecked("contract1");

    let config: ConfigResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &QueryMsg::Config {})
        .unwrap();

    // default start_trading_time is None
    assert!(config.config.start_trading_time.is_none());

    // update start_trading_time
    let update_trading_time_msg =
        ExecuteMsg::UpdateStartTradingTime(Some(Timestamp::from_nanos(1_000_000_000)));
    let res = router.execute_contract(creator, minter_addr.clone(), &update_trading_time_msg, &[]);
    assert!(res.is_ok());

    let config: ConfigResponse = router
        .wrap()
        .query_wasm_smart(minter_addr, &QueryMsg::Config {})
        .unwrap();

    if let Some(_) = config.config.start_trading_time {
        assert_eq!(
            config.config.start_trading_time,
            Some(Timestamp::from_nanos(0).plus_nanos(1_000_000_000))
        );
    } else {
        panic!("start_trading_time not found");
    }
}
