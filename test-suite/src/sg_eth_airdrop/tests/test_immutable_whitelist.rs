use crate::common_setup::contract_boxes::custom_mock_app;
use crate::sg_eth_airdrop::constants::claim_constants::{
    CONFIG_PLAINTEXT, MOCK_AIRDROP_ADDR_STR, MOCK_MINTER_ADDR_STR, OWNER,
};
use crate::sg_eth_airdrop::constants::collection_constants::WHITELIST_AMOUNT;
use crate::sg_eth_airdrop::setup::configure_mock_minter::configure_mock_minter_with_mock_whitelist;
use crate::sg_eth_airdrop::setup::execute_msg::instantiate_contract;
use crate::sg_eth_airdrop::setup::test_msgs::InstantiateParams;

use cosmwasm_std::Addr;
use sg_eth_airdrop::contract::INSTANTIATION_FEE;
use sg_eth_airdrop::msg::QueryMsg;
use whitelist_immutable::helpers::WhitelistImmutableContract;

#[test]
fn test_instantiate_with_addresses() {
    let addresses: Vec<String> = vec![
        "addr1".to_string(),
        "addr2".to_string(),
        "addr3".to_string(),
    ];

    let mut app = { custom_mock_app }();
    configure_mock_minter_with_mock_whitelist(&mut app);
    let minter_addr = Addr::unchecked(MOCK_MINTER_ADDR_STR);
    let airdrop_contract = Addr::unchecked(MOCK_AIRDROP_ADDR_STR);

    let params = InstantiateParams {
        addresses,
        funds_amount: WHITELIST_AMOUNT + INSTANTIATION_FEE,
        expected_airdrop_contract_id: 4,
        minter_address: minter_addr,
        admin_account: Addr::unchecked(OWNER),
        app: &mut app,
        per_address_limit: 1,
        claim_msg_plaintext: CONFIG_PLAINTEXT.to_string(),
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
    let minter_addr = Addr::unchecked(MOCK_MINTER_ADDR_STR);

    let params = InstantiateParams {
        addresses,
        funds_amount: WHITELIST_AMOUNT + INSTANTIATION_FEE,
        expected_airdrop_contract_id: 4,
        minter_address: minter_addr,
        admin_account: Addr::unchecked(OWNER),
        app: &mut app,
        per_address_limit: 20,
        claim_msg_plaintext: CONFIG_PLAINTEXT.to_string(),
    };
    instantiate_contract(params).unwrap();
    let whitelist_immutable = Addr::unchecked("contract4");
    let res: u32 = WhitelistImmutableContract(whitelist_immutable)
        .per_address_limit(&app.wrap())
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
    let minter_addr = Addr::unchecked(MOCK_MINTER_ADDR_STR);

    let params = InstantiateParams {
        addresses,
        funds_amount: WHITELIST_AMOUNT + INSTANTIATION_FEE,
        expected_airdrop_contract_id: 4,
        minter_address: minter_addr,
        admin_account: Addr::unchecked(OWNER),
        app: &mut app,
        per_address_limit: 20,
        claim_msg_plaintext: CONFIG_PLAINTEXT.to_string(),
    };
    instantiate_contract(params).unwrap();
    let whitelist_immutable = Addr::unchecked("contract4");
    let res: u64 = WhitelistImmutableContract(whitelist_immutable)
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
    let minter_addr = Addr::unchecked(MOCK_MINTER_ADDR_STR);

    let params = InstantiateParams {
        addresses,
        funds_amount: WHITELIST_AMOUNT + INSTANTIATION_FEE,
        expected_airdrop_contract_id: 4,
        minter_address: minter_addr,
        admin_account: Addr::unchecked(OWNER),
        app: &mut app,
        per_address_limit: 20,
        claim_msg_plaintext: CONFIG_PLAINTEXT.to_string(),
    };
    instantiate_contract(params).unwrap();
    let whitelist_immutable = Addr::unchecked("contract4");
    let res: bool = WhitelistImmutableContract(whitelist_immutable.clone())
        .includes(&app.wrap(), "addr3".to_string())
        .unwrap();
    assert!(res);

    let res: bool = WhitelistImmutableContract(whitelist_immutable)
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
    let minter_addr = Addr::unchecked(MOCK_MINTER_ADDR_STR);

    let params = InstantiateParams {
        addresses,
        funds_amount: WHITELIST_AMOUNT + INSTANTIATION_FEE,
        expected_airdrop_contract_id: 4,
        minter_address: minter_addr,
        admin_account: Addr::unchecked(OWNER),
        app: &mut app,
        per_address_limit: 20,
        claim_msg_plaintext: CONFIG_PLAINTEXT.to_string(),
    };
    instantiate_contract(params).unwrap();
    let whitelist_immutable = Addr::unchecked("contract4");
    let res: whitelist_immutable::state::Config = WhitelistImmutableContract(whitelist_immutable)
        .config(&app.wrap())
        .unwrap();
    let expected_config = whitelist_immutable::state::Config {
        admin: Addr::unchecked("contract3"),
        per_address_limit: 20,
        mint_discount_bps: Some(0),
    };
    assert_eq!(res, expected_config);
}
