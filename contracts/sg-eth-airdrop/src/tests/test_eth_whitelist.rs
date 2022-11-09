use crate::msg::{ExecuteMsg, QueryMsg};
use crate::tests_folder::constants::{AIRDROP_CONTRACT, OWNER};
use crate::tests_folder::shared::get_instantiate_contract;
use cosmwasm_std::Addr;
use cw_multi_test::Executor;

#[test]
fn test_instantiate_with_addresses() {
    let addresses: Vec<String> = vec![
        "addr1".to_string(),
        "addr2".to_string(),
        "addr3".to_string(),
    ];
    let app = get_instantiate_contract(addresses, 10000);
    let sg_eth_addr = Addr::unchecked(AIRDROP_CONTRACT);

    let query_msg = QueryMsg::AirdropEligible {
        eth_address: "addr1".to_string(),
    };
    let result: bool = app
        .wrap()
        .query_wasm_smart(sg_eth_addr.clone(), &query_msg)
        .unwrap();
    assert!(result);

    let query_msg = QueryMsg::AirdropEligible {
        eth_address: "lies".to_string(),
    };
    let result: bool = app
        .wrap()
        .query_wasm_smart(sg_eth_addr, &query_msg)
        .unwrap();
    assert!(!result);
}

#[test]
fn test_not_authorized_add_eth() {
    let mut app = get_instantiate_contract(vec![], 10000);

    let fake_admin = Addr::unchecked("fake_admin");
    let sg_eth_addr = Addr::unchecked(AIRDROP_CONTRACT);
    let eth_address = Addr::unchecked("testing_addr");
    let execute_msg = ExecuteMsg::AddEligibleEth {
        eth_addresses: vec![eth_address.to_string()],
    };
    let res = app.execute_contract(fake_admin, sg_eth_addr, &execute_msg, &[]);
    let error = res.unwrap_err();
    let expected_err_msg = "Unauthorized admin, sender is fake_admin";
    assert_eq!(error.root_cause().to_string(), expected_err_msg)
}

#[test]
fn test_authorized_add_eth() {
    let mut app = get_instantiate_contract(vec![], 10000);
    let sg_eth_addr = Addr::unchecked(AIRDROP_CONTRACT);

    let eth_address = Addr::unchecked("testing_addr");
    let execute_msg = ExecuteMsg::AddEligibleEth {
        eth_addresses: vec![eth_address.to_string()],
    };
    let owner_admin = Addr::unchecked(OWNER);
    let res = app.execute_contract(owner_admin, sg_eth_addr, &execute_msg, &[]);
    res.unwrap();
}

#[test]
fn test_add_eth_and_verify() {
    let mut app = get_instantiate_contract(vec![], 10000);
    let sg_eth_addr = Addr::unchecked(AIRDROP_CONTRACT);

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
        .query_wasm_smart(sg_eth_addr.clone(), &query_msg)
        .unwrap();
    assert!(!result);

    let owner_admin = Addr::unchecked(OWNER);
    let _ = app.execute_contract(owner_admin, sg_eth_addr.clone(), &execute_msg, &[]);

    //test after add
    let query_msg = QueryMsg::AirdropEligible {
        eth_address: eth_address_str,
    };
    let result: bool = app
        .wrap()
        .query_wasm_smart(sg_eth_addr, &query_msg)
        .unwrap();
    assert!(result);
}
