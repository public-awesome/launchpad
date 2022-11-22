use crate::msg::{ExecuteMsg, QueryMsg};
use crate::tests_folder::constants::{
    MOCK_AIRDROP_ADDR_STR, MOCK_MINTER_ADDR_STR, OWNER, WHITELIST_AMOUNT,
};
use crate::tests_folder::tests_setup::{
    configure_mock_minter_with_mock_whitelist, custom_mock_app, instantiate_contract,
    InstantiateParams,
};
use cosmwasm_std::Addr;
use cw_multi_test::Executor;

#[test]
fn test_instantiate_with_addresses() {
    let addresses: Vec<String> = vec![
        "addr1".to_string(),
        "addr2".to_string(),
        "addr3".to_string(),
    ];

    let mut app = custom_mock_app();
    configure_mock_minter_with_mock_whitelist(&mut app);
    let minter_addr = Addr::unchecked(MOCK_MINTER_ADDR_STR);
    let airdrop_contract = Addr::unchecked(MOCK_AIRDROP_ADDR_STR);

    let params = InstantiateParams {
        addresses,
        funds_amount: WHITELIST_AMOUNT,
        expected_airdrop_contract_id: 4,
        minter_address: minter_addr,
        admin_account: Addr::unchecked(OWNER),
        app: &mut app,
        per_address_limit: 1,
    };
    instantiate_contract(params);

    let query_msg = QueryMsg::AirdropEligible {
        eth_address: "addr1".to_string(),
    };
    let result: bool = app
        .wrap()
        .query_wasm_smart(airdrop_contract.clone(), &query_msg)
        .unwrap();
    assert!(result);

    let query_msg = QueryMsg::AirdropEligible {
        eth_address: "lies".to_string(),
    };
    let result: bool = app
        .wrap()
        .query_wasm_smart(airdrop_contract, &query_msg)
        .unwrap();
    assert!(!result);
}

#[test]
fn test_not_authorized_add_eth() {
    let mut app = custom_mock_app();
    configure_mock_minter_with_mock_whitelist(&mut app);
    let minter_addr = Addr::unchecked(MOCK_MINTER_ADDR_STR);
    let airdrop_contract = Addr::unchecked(MOCK_AIRDROP_ADDR_STR);

    let params = InstantiateParams {
        addresses: vec![],
        funds_amount: WHITELIST_AMOUNT,
        expected_airdrop_contract_id: 4,
        minter_address: minter_addr,
        admin_account: Addr::unchecked(OWNER),
        app: &mut app,
        per_address_limit: 1,
    };
    instantiate_contract(params);

    let fake_admin = Addr::unchecked("fake_admin");
    let eth_address = Addr::unchecked("testing_addr");
    let execute_msg = ExecuteMsg::AddEligibleEth {
        eth_addresses: vec![eth_address.to_string()],
    };
    let res = app.execute_contract(fake_admin, airdrop_contract, &execute_msg, &[]);
    let error = res.unwrap_err();
    let expected_err_msg = "Unauthorized admin, sender is fake_admin";
    assert_eq!(error.root_cause().to_string(), expected_err_msg)
}

#[test]
fn test_authorized_add_eth() {
    let mut app = custom_mock_app();
    configure_mock_minter_with_mock_whitelist(&mut app);
    let minter_addr = Addr::unchecked(MOCK_MINTER_ADDR_STR);
    let airdrop_contract = Addr::unchecked(MOCK_AIRDROP_ADDR_STR);

    let params = InstantiateParams {
        addresses: vec![],
        funds_amount: WHITELIST_AMOUNT,
        expected_airdrop_contract_id: 4,
        minter_address: minter_addr,
        admin_account: Addr::unchecked(OWNER),
        app: &mut app,
        per_address_limit: 1,
    };
    instantiate_contract(params);

    let eth_address = Addr::unchecked("testing_addr");
    let execute_msg = ExecuteMsg::AddEligibleEth {
        eth_addresses: vec![eth_address.to_string()],
    };
    let owner_admin = Addr::unchecked(OWNER);
    let res = app.execute_contract(owner_admin, airdrop_contract, &execute_msg, &[]);
    res.unwrap();
}

#[test]
fn test_add_eth_and_verify() {
    let mut app = custom_mock_app();
    configure_mock_minter_with_mock_whitelist(&mut app);
    let minter_addr = Addr::unchecked(MOCK_MINTER_ADDR_STR);
    let airdrop_contract = Addr::unchecked(MOCK_AIRDROP_ADDR_STR);

    let params = InstantiateParams {
        addresses: vec![],
        funds_amount: WHITELIST_AMOUNT,
        expected_airdrop_contract_id: 4,
        minter_address: minter_addr,
        admin_account: Addr::unchecked(OWNER),
        app: &mut app,
        per_address_limit: 1,
    };
    instantiate_contract(params);

    let eth_address_str = Addr::unchecked("testing_addr").to_string();
    let execute_msg = ExecuteMsg::AddEligibleEth {
        eth_addresses: vec![eth_address_str.clone()],
    };

    // test before add:
    let query_msg = QueryMsg::AirdropEligible {
        eth_address: eth_address_str.clone(),
    };
    let result: bool = app
        .wrap()
        .query_wasm_smart(airdrop_contract.clone(), &query_msg)
        .unwrap();
    assert!(!result);

    let owner_admin = Addr::unchecked(OWNER);
    let _ = app.execute_contract(owner_admin, airdrop_contract.clone(), &execute_msg, &[]);

    //test after add
    let query_msg = QueryMsg::AirdropEligible {
        eth_address: eth_address_str,
    };
    let result: bool = app
        .wrap()
        .query_wasm_smart(airdrop_contract, &query_msg)
        .unwrap();
    assert!(result);
}
