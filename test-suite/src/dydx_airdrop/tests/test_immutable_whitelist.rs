use crate::common_setup::contract_boxes::custom_mock_app;
use crate::dydx_airdrop::constants::claim_constants::{CONFIG_PLAINTEXT, MOCK_AIRDROP_ADDR_STR, MOCK_MINTER_ADDR_STR, MOCK_NAME_COLLECTION_ADDR, MOCK_NAME_DISCOUNT_WL_ADDR_STR, OWNER};
use crate::dydx_airdrop::constants::collection_constants::WHITELIST_AMOUNT;
use crate::dydx_airdrop::setup::configure_mock_minter::configure_mock_minter_with_mock_whitelist;
use crate::dydx_airdrop::setup::execute_msg::instantiate_contract;
use crate::dydx_airdrop::setup::test_msgs::InstantiateParams;

use cosmwasm_std::Addr;
use dydx_airdrop::contract::INSTANTIATION_FEE;
use dydx_airdrop::msg::QueryMsg;
use whitelist_immutable_flex::helpers::WhitelistImmutableFlexContract;
use whitelist_immutable_flex::msg::Member;
use whitelist_immutable_flex::state::Config;

#[test]
fn test_instantiate_with_addresses() {
    let addresses: Vec<String> = vec![
        "addr1".to_string(),
        "addr2".to_string(),
        "addr3".to_string(),
    ];

    let mut app = { custom_mock_app }();
    configure_mock_minter_with_mock_whitelist(&mut app);
    let minter_address = MOCK_MINTER_ADDR_STR.to_string();
    let name_discount_wl_address = MOCK_NAME_DISCOUNT_WL_ADDR_STR.to_string();
    let name_collection_address = MOCK_NAME_COLLECTION_ADDR.to_string();
    let airdrop_contract = Addr::unchecked(MOCK_AIRDROP_ADDR_STR);

    let params = InstantiateParams {
        members: vec![Member {address: addresses[0].clone(), mint_count: 1}, Member {address: addresses[1].clone(), mint_count: 1}, Member {address: addresses[2].clone(), mint_count: 1}],
        funds_amount: WHITELIST_AMOUNT + INSTANTIATION_FEE,
        expected_airdrop_contract_id: 4,
        minter_address,
        admin_account: OWNER.to_string(),
        app: &mut app,
        claim_msg_plaintext: CONFIG_PLAINTEXT.to_string(),
        name_discount_wl_address,
        name_collection_address,
        airdrop_count_limit: 500,
        airdrop_amount: WHITELIST_AMOUNT,
    };
    instantiate_contract(params).unwrap();

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
fn test_whitelist_immutable_address_limit() {
    let addresses: Vec<String> = vec![
        "addr1".to_string(),
        "addr2".to_string(),
        "addr3".to_string(),
    ];

    let mut app = custom_mock_app();
    configure_mock_minter_with_mock_whitelist(&mut app);
    let minter_address = MOCK_MINTER_ADDR_STR.to_string();
    let name_discount_wl_address = MOCK_NAME_DISCOUNT_WL_ADDR_STR.to_string();
    let name_collection_address = MOCK_NAME_COLLECTION_ADDR.to_string();

    let params = InstantiateParams {
        members: vec![Member {address: addresses[0].clone(), mint_count: 20}, Member {address: addresses[1].clone(), mint_count: 1}, Member {address: addresses[2].clone(), mint_count: 1}],
        funds_amount: WHITELIST_AMOUNT + INSTANTIATION_FEE,
        expected_airdrop_contract_id: 4,
        minter_address,
        admin_account: OWNER.to_string(),
        app: &mut app,
        claim_msg_plaintext: CONFIG_PLAINTEXT.to_string(),
        name_discount_wl_address,
        name_collection_address,
        airdrop_count_limit: 500,
        airdrop_amount: WHITELIST_AMOUNT,
    };
    instantiate_contract(params).unwrap();
    let whitelist_immutable_flex = Addr::unchecked("contract4");
    let res: u32 = WhitelistImmutableFlexContract(whitelist_immutable_flex)
        .mint_count(&app.wrap(), "addr1".to_string())
        .unwrap();
    assert_eq!(res, 20);
}

#[test]
fn test_whitelist_immutable_address_count() {
    let addresses: Vec<String> = vec![
        "addr1".to_string(),
        "addr2".to_string(),
        "addr3".to_string(),
    ];

    let mut app = custom_mock_app();
    configure_mock_minter_with_mock_whitelist(&mut app);
    let minter_address = MOCK_MINTER_ADDR_STR.to_string();
    let name_discount_wl_address = MOCK_NAME_DISCOUNT_WL_ADDR_STR.to_string();
    let name_collection_address = MOCK_NAME_COLLECTION_ADDR.to_string();

    let params = InstantiateParams {
        members: vec![Member {address: addresses[0].clone(), mint_count: 1}, Member {address: addresses[1].clone(), mint_count: 1}, Member {address: addresses[2].clone(), mint_count: 1}],
        funds_amount: WHITELIST_AMOUNT + INSTANTIATION_FEE,
        expected_airdrop_contract_id: 4,
        minter_address,
        admin_account: OWNER.to_string(),
        app: &mut app,
        claim_msg_plaintext: CONFIG_PLAINTEXT.to_string(),
        name_discount_wl_address,
        name_collection_address,
        airdrop_count_limit: 500,
        airdrop_amount: WHITELIST_AMOUNT,
    };
    instantiate_contract(params).unwrap();
    let whitelist_immutable = Addr::unchecked("contract4");
    let res: u64 = WhitelistImmutableFlexContract(whitelist_immutable)
        .address_count(&app.wrap())
        .unwrap();
    assert_eq!(res, 3);
}

#[test]
fn test_whitelist_immutable_address_includes() {
    let addresses: Vec<String> = vec![
        "addr1".to_string(),
        "addr2".to_string(),
        "addr3".to_string(),
    ];

    let mut app = custom_mock_app();
    configure_mock_minter_with_mock_whitelist(&mut app);
    let minter_address = MOCK_MINTER_ADDR_STR.to_string();
    let name_discount_wl_address = MOCK_NAME_DISCOUNT_WL_ADDR_STR.to_string();
    let name_collection_address = MOCK_NAME_COLLECTION_ADDR.to_string();

    let params = InstantiateParams {
        members: vec![Member {address: addresses[0].clone(), mint_count: 1}, Member {address: addresses[1].clone(), mint_count: 1}, Member {address: addresses[2].clone(), mint_count: 1}],
        funds_amount: WHITELIST_AMOUNT + INSTANTIATION_FEE,
        expected_airdrop_contract_id: 4,
        minter_address,
        admin_account: OWNER.to_string(),
        app: &mut app,
        claim_msg_plaintext: CONFIG_PLAINTEXT.to_string(),
        name_discount_wl_address,
        name_collection_address,
        airdrop_count_limit: 500,
        airdrop_amount: WHITELIST_AMOUNT,
    };
    instantiate_contract(params).unwrap();
    let whitelist_immutable = Addr::unchecked("contract4");
    let res: bool = WhitelistImmutableFlexContract(whitelist_immutable.clone())
        .includes(&app.wrap(), "addr3".to_string())
        .unwrap();
    assert!(res);

    let res: bool = WhitelistImmutableFlexContract(whitelist_immutable)
        .includes(&app.wrap(), "nonsense".to_string())
        .unwrap();
    assert!(!res);
}

#[test]
fn test_whitelist_immutable_address_config() {
    let addresses: Vec<String> = vec![
        "addr1".to_string(),
        "addr2".to_string(),
        "addr3".to_string(),
    ];

    let mut app = custom_mock_app();
    configure_mock_minter_with_mock_whitelist(&mut app);
    let minter_address = MOCK_MINTER_ADDR_STR.to_string();
    let name_discount_wl_address = MOCK_NAME_DISCOUNT_WL_ADDR_STR.to_string();
    let name_collection_address = MOCK_NAME_COLLECTION_ADDR.to_string();

    let params = InstantiateParams {
        members: vec![Member {address: addresses[0].clone(), mint_count: 1}, Member {address: addresses[1].clone(), mint_count: 1}, Member {address: addresses[2].clone(), mint_count: 1}],
        funds_amount: WHITELIST_AMOUNT + INSTANTIATION_FEE,
        expected_airdrop_contract_id: 4,
        minter_address,
        admin_account: OWNER.to_string(),
        app: &mut app,
        claim_msg_plaintext: CONFIG_PLAINTEXT.to_string(),
        name_discount_wl_address,
        name_collection_address,
        airdrop_count_limit: 500,
        airdrop_amount: WHITELIST_AMOUNT,
    };
    instantiate_contract(params).unwrap();
    let whitelist_immutable = Addr::unchecked("contract4");
    let res: Config = WhitelistImmutableFlexContract(whitelist_immutable)
        .config(&app.wrap())
        .unwrap();
    let expected_config = whitelist_immutable_flex::state::Config {
        admin: Addr::unchecked("contract3"),
        mint_discount_bps: Some(0),
    };
    assert_eq!(res, expected_config);
}
