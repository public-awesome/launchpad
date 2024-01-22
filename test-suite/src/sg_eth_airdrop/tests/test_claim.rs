use crate::common_setup::contract_boxes::{custom_mock_app, App};
use crate::sg_eth_airdrop::constants::collection_constants::WHITELIST_AMOUNT;
use crate::sg_eth_airdrop::setup::configure_mock_minter::configure_mock_minter_with_mock_whitelist;
use crate::sg_eth_airdrop::setup::setup_signatures::{
    get_msg_plaintext, get_signature, get_wallet_and_sig,
};
use crate::sg_eth_airdrop::setup::test_msgs::InstantiateParams;
use async_std::task;
use cosmwasm_std::{Addr, Attribute, Coin, Uint128};
use sg_eth_airdrop::msg::{ExecuteMsg, QueryMsg};

use ethers_core::rand::thread_rng;
use ethers_signers::{LocalWallet, Signer};

use crate::sg_eth_airdrop::constants::claim_constants::{
    CONFIG_PLAINTEXT, MOCK_AIRDROP_ADDR_STR, MOCK_MINTER_ADDR_STR, NATIVE_DENOM, OWNER,
    STARGAZE_WALLET_01, STARGAZE_WALLET_02,
};

use crate::sg_eth_airdrop::setup::execute_msg::{
    execute_contract_error_with_msg, execute_contract_with_msg, instantiate_contract,
};

use sg_eth_airdrop::contract::INSTANTIATION_FEE;

fn query_minter_as_expected(app: &mut App, airdrop_contract: Addr, minter_addr: Addr) {
    let query_msg = QueryMsg::GetMinter {};
    let result: Addr = app
        .wrap()
        .query_wasm_smart(airdrop_contract, &query_msg)
        .unwrap();
    assert_eq!(minter_addr, result);
}

#[test]
fn test_instantiate() {
    let mut app = custom_mock_app();
    let minter_address = Addr::unchecked("contract1");
    let claim_plaintext = &get_msg_plaintext(STARGAZE_WALLET_01.to_string());
    let (_, _, _, eth_addr_str) = get_wallet_and_sig(claim_plaintext.clone());
    let params = InstantiateParams {
        addresses: vec![eth_addr_str],
        funds_amount: WHITELIST_AMOUNT + INSTANTIATION_FEE,
        expected_airdrop_contract_id: 1,
        minter_address,
        admin_account: Addr::unchecked(OWNER),
        app: &mut app,
        per_address_limit: 1,
        claim_msg_plaintext: CONFIG_PLAINTEXT.to_string(),
    };
    instantiate_contract(params).unwrap();
}

#[test]
fn test_instantiate_plaintext_too_long() {
    let long_config_plaintext: String = String::from_utf8(vec![b'X'; 1001]).unwrap() + " {wallet}";
    let mut app = custom_mock_app();
    let minter_address = Addr::unchecked("contract1");
    let claim_plaintext = &get_msg_plaintext(STARGAZE_WALLET_01.to_string());
    let (_, _, _, eth_addr_str) = get_wallet_and_sig(claim_plaintext.clone());
    let params = InstantiateParams {
        addresses: vec![eth_addr_str],
        funds_amount: WHITELIST_AMOUNT + INSTANTIATION_FEE,
        expected_airdrop_contract_id: 1,
        minter_address,
        admin_account: Addr::unchecked(OWNER),
        app: &mut app,
        per_address_limit: 1,
        claim_msg_plaintext: long_config_plaintext,
    };
    let res = instantiate_contract(params).unwrap_err();
    assert_eq!(
        res.root_cause().to_string(),
        "Plaintext message is too long"
    );
}

#[test]
fn test_instantiate_plaintext_missing_wallet() {
    let plaintext_config_no_wallet = "This message doesn't have wallet string".to_string();
    let mut app = custom_mock_app();
    let minter_address = Addr::unchecked("contract1");
    let claim_plaintext = &get_msg_plaintext(STARGAZE_WALLET_01.to_string());
    let (_, _, _, eth_addr_str) = get_wallet_and_sig(claim_plaintext.clone());
    let params = InstantiateParams {
        addresses: vec![eth_addr_str],
        funds_amount: WHITELIST_AMOUNT + INSTANTIATION_FEE,
        expected_airdrop_contract_id: 1,
        minter_address,
        admin_account: Addr::unchecked(OWNER),
        app: &mut app,
        per_address_limit: 1,
        claim_msg_plaintext: plaintext_config_no_wallet,
    };
    let res = instantiate_contract(params).unwrap_err();
    assert_eq!(
        res.root_cause().to_string(),
        "Plaintext message must contain `{wallet}` string"
    );
}

#[test]
fn test_airdrop_eligible_query() {
    let claim_plaintext = &get_msg_plaintext(STARGAZE_WALLET_01.to_string());
    let (_, _, _, eth_addr_str) = get_wallet_and_sig(claim_plaintext.clone());
    let mut app = custom_mock_app();
    configure_mock_minter_with_mock_whitelist(&mut app);
    let minter_addr = Addr::unchecked(MOCK_MINTER_ADDR_STR);
    let airdrop_contract = Addr::unchecked(MOCK_AIRDROP_ADDR_STR);

    let params = InstantiateParams {
        addresses: vec![eth_addr_str.clone()],
        funds_amount: WHITELIST_AMOUNT + INSTANTIATION_FEE,
        expected_airdrop_contract_id: 4,
        minter_address: minter_addr.clone(),
        admin_account: Addr::unchecked(OWNER),
        app: &mut app,
        per_address_limit: 1,
        claim_msg_plaintext: CONFIG_PLAINTEXT.to_string(),
    };
    instantiate_contract(params).unwrap();
    query_minter_as_expected(&mut app, airdrop_contract.clone(), minter_addr);
    let query_msg = QueryMsg::AirdropEligible {
        eth_address: eth_addr_str,
    };
    let result: bool = app
        .wrap()
        .query_wasm_smart(airdrop_contract.clone(), &query_msg)
        .unwrap();
    assert!(result);

    let query_msg = QueryMsg::AirdropEligible {
        eth_address: "0x-some-fake-address".to_string(),
    };
    let result: bool = app
        .wrap()
        .query_wasm_smart(airdrop_contract, &query_msg)
        .unwrap();
    assert!(!result);
}

#[test]
fn test_valid_eth_sig_claim() {
    let claim_plaintext = &get_msg_plaintext(STARGAZE_WALLET_01.to_string());
    let (_, eth_sig_str, _, eth_addr_str) = get_wallet_and_sig(claim_plaintext.clone());
    let mut app = custom_mock_app();
    configure_mock_minter_with_mock_whitelist(&mut app);
    let minter_addr = Addr::unchecked(MOCK_MINTER_ADDR_STR);
    let airdrop_contract = Addr::unchecked(MOCK_AIRDROP_ADDR_STR);

    let params = InstantiateParams {
        addresses: vec![eth_addr_str.clone()],
        funds_amount: WHITELIST_AMOUNT + INSTANTIATION_FEE,
        expected_airdrop_contract_id: 4,
        minter_address: minter_addr.clone(),
        admin_account: Addr::unchecked(OWNER),
        app: &mut app,
        per_address_limit: 1,
        claim_msg_plaintext: CONFIG_PLAINTEXT.to_string(),
    };
    instantiate_contract(params).unwrap();
    query_minter_as_expected(&mut app, airdrop_contract.clone(), minter_addr);
    let claim_message = ExecuteMsg::ClaimAirdrop {
        eth_address: eth_addr_str,
        eth_sig: eth_sig_str,
    };
    let stargaze_wallet_01 = Addr::unchecked(STARGAZE_WALLET_01);

    let res = execute_contract_with_msg(
        claim_message,
        &mut app,
        stargaze_wallet_01,
        airdrop_contract.clone(),
    )
    .unwrap();
    let expected_attributes = [
        Attribute {
            key: "_contract_address".to_string(),
            value: airdrop_contract.to_string(),
        },
        Attribute {
            key: "claimed_amount".to_string(),
            value: WHITELIST_AMOUNT.to_string(),
        },
    ];
    assert_eq!(res.events[1].attributes, expected_attributes);
}

#[test]
fn test_invalid_eth_sig_claim() {
    let claim_plaintext = &get_msg_plaintext(STARGAZE_WALLET_01.to_string());
    let (_, _, _, eth_addr_str) = get_wallet_and_sig(claim_plaintext.clone());
    let (_, eth_sig_str_2, _, _) = get_wallet_and_sig(claim_plaintext.clone());

    let mut app = custom_mock_app();
    configure_mock_minter_with_mock_whitelist(&mut app);
    let minter_addr = Addr::unchecked(MOCK_MINTER_ADDR_STR);
    let airdrop_contract = Addr::unchecked(MOCK_AIRDROP_ADDR_STR);

    let params = InstantiateParams {
        addresses: vec![eth_addr_str.clone()],
        funds_amount: WHITELIST_AMOUNT + INSTANTIATION_FEE,
        expected_airdrop_contract_id: 4,
        minter_address: minter_addr.clone(),
        admin_account: Addr::unchecked(OWNER),
        app: &mut app,
        per_address_limit: 1,
        claim_msg_plaintext: CONFIG_PLAINTEXT.to_string(),
    };
    instantiate_contract(params).unwrap();
    query_minter_as_expected(&mut app, airdrop_contract.clone(), minter_addr);
    let claim_message = ExecuteMsg::ClaimAirdrop {
        eth_address: eth_addr_str.clone(),
        eth_sig: eth_sig_str_2,
    };
    let stargaze_wallet_01 = Addr::unchecked(STARGAZE_WALLET_01);
    let res = execute_contract_error_with_msg(
        claim_message,
        &mut app,
        stargaze_wallet_01,
        airdrop_contract,
    );
    assert_eq!(res, format!("Address {eth_addr_str} is not eligible"));
}

#[test]
fn test_can_not_claim_twice() {
    let claim_plaintext = &get_msg_plaintext(STARGAZE_WALLET_01.to_string());
    let (_, eth_sig_str, _, eth_addr_str) = get_wallet_and_sig(claim_plaintext.clone());

    let mut app = custom_mock_app();
    configure_mock_minter_with_mock_whitelist(&mut app);
    let minter_addr = Addr::unchecked(MOCK_MINTER_ADDR_STR);
    let airdrop_contract = Addr::unchecked(MOCK_AIRDROP_ADDR_STR);

    let params = InstantiateParams {
        addresses: vec![eth_addr_str.clone()],
        funds_amount: WHITELIST_AMOUNT + INSTANTIATION_FEE,
        expected_airdrop_contract_id: 4,
        minter_address: minter_addr.clone(),
        admin_account: Addr::unchecked(OWNER),
        app: &mut app,
        per_address_limit: 1,
        claim_msg_plaintext: CONFIG_PLAINTEXT.to_string(),
    };
    instantiate_contract(params).unwrap();
    query_minter_as_expected(&mut app, airdrop_contract.clone(), minter_addr);
    let claim_message = ExecuteMsg::ClaimAirdrop {
        eth_address: eth_addr_str.clone(),
        eth_sig: eth_sig_str,
    };
    let stargaze_wallet_01 = Addr::unchecked(STARGAZE_WALLET_01);
    let res = execute_contract_with_msg(
        claim_message.clone(),
        &mut app,
        stargaze_wallet_01.clone(),
        airdrop_contract.clone(),
    )
    .unwrap();

    let expected_attributes = [
        Attribute {
            key: "_contract_address".to_string(),
            value: airdrop_contract.to_string(),
        },
        Attribute {
            key: "claimed_amount".to_string(),
            value: WHITELIST_AMOUNT.to_string(),
        },
    ];
    assert_eq!(res.events[1].attributes, expected_attributes);
    let res = execute_contract_error_with_msg(
        claim_message,
        &mut app,
        stargaze_wallet_01,
        airdrop_contract,
    );
    let expected_error = format!("Address {eth_addr_str} has already claimed all available mints");
    assert_eq!(res, expected_error);
}

#[test]
fn test_claim_one_valid_airdrop() {
    let claim_plaintext = &get_msg_plaintext(STARGAZE_WALLET_01.to_string());
    let (_, eth_sig_str, _, eth_addr_str) = get_wallet_and_sig(claim_plaintext.clone());

    let stargaze_wallet_01 = Addr::unchecked(STARGAZE_WALLET_01);

    let mut app = custom_mock_app();
    configure_mock_minter_with_mock_whitelist(&mut app);
    let minter_addr = Addr::unchecked(MOCK_MINTER_ADDR_STR);
    let airdrop_contract = Addr::unchecked(MOCK_AIRDROP_ADDR_STR);

    let params = InstantiateParams {
        addresses: vec![eth_addr_str.clone()],
        funds_amount: WHITELIST_AMOUNT + INSTANTIATION_FEE,
        expected_airdrop_contract_id: 4,
        minter_address: minter_addr.clone(),
        admin_account: Addr::unchecked(OWNER),
        app: &mut app,
        per_address_limit: 1,
        claim_msg_plaintext: CONFIG_PLAINTEXT.to_string(),
    };
    instantiate_contract(params).unwrap();
    query_minter_as_expected(&mut app, airdrop_contract.clone(), minter_addr);
    let balances = app
        .wrap()
        .query_all_balances(stargaze_wallet_01.clone())
        .unwrap();
    assert_eq!(balances, []);

    let claim_message = ExecuteMsg::ClaimAirdrop {
        eth_address: eth_addr_str,
        eth_sig: eth_sig_str,
    };
    let _ = execute_contract_with_msg(
        claim_message,
        &mut app,
        stargaze_wallet_01.clone(),
        airdrop_contract,
    )
    .unwrap();

    let balances = app.wrap().query_all_balances(stargaze_wallet_01).unwrap();
    let expected_balance = [Coin {
        denom: NATIVE_DENOM.to_string(),
        amount: Uint128::new(WHITELIST_AMOUNT),
    }];
    assert_eq!(balances, expected_balance)
}

#[test]
fn test_claim_twice_receive_funds_once() {
    let claim_plaintext = &get_msg_plaintext(STARGAZE_WALLET_01.to_string());
    let (_, eth_sig_str, _, eth_addr_str) = get_wallet_and_sig(claim_plaintext.clone());

    let stargaze_wallet_01 = Addr::unchecked(STARGAZE_WALLET_01);
    let mut app = custom_mock_app();

    configure_mock_minter_with_mock_whitelist(&mut app);
    let minter_addr = Addr::unchecked(MOCK_MINTER_ADDR_STR);
    let airdrop_contract = Addr::unchecked(MOCK_AIRDROP_ADDR_STR);

    let params = InstantiateParams {
        addresses: vec![eth_addr_str.clone()],
        funds_amount: WHITELIST_AMOUNT + INSTANTIATION_FEE,
        expected_airdrop_contract_id: 4,
        minter_address: minter_addr.clone(),
        admin_account: Addr::unchecked(OWNER),
        app: &mut app,
        per_address_limit: 1,
        claim_msg_plaintext: CONFIG_PLAINTEXT.to_string(),
    };
    instantiate_contract(params).unwrap();
    query_minter_as_expected(&mut app, airdrop_contract.clone(), minter_addr);
    let balances = app
        .wrap()
        .query_all_balances(stargaze_wallet_01.clone())
        .unwrap();
    assert_eq!(balances, []);

    let claim_message = ExecuteMsg::ClaimAirdrop {
        eth_address: eth_addr_str.clone(),
        eth_sig: eth_sig_str,
    };
    let _ = execute_contract_with_msg(
        claim_message.clone(),
        &mut app,
        stargaze_wallet_01.clone(),
        airdrop_contract.clone(),
    )
    .unwrap();

    let balances = app
        .wrap()
        .query_all_balances(stargaze_wallet_01.clone())
        .unwrap();
    let expected_balance = [Coin {
        denom: NATIVE_DENOM.to_string(),
        amount: Uint128::new(WHITELIST_AMOUNT),
    }];
    assert_eq!(balances, expected_balance);
    let res = execute_contract_error_with_msg(
        claim_message,
        &mut app,
        stargaze_wallet_01.clone(),
        airdrop_contract,
    );
    let expected_error = format!("Address {eth_addr_str} has already claimed all available mints");
    assert_eq!(res, expected_error);
    let balances = app.wrap().query_all_balances(stargaze_wallet_01).unwrap();
    let expected_balance = [Coin {
        denom: NATIVE_DENOM.to_string(),
        amount: Uint128::new(WHITELIST_AMOUNT),
    }];
    assert_eq!(balances, expected_balance);
}

#[test]
fn test_ineligible_does_not_receive_funds() {
    let claim_plaintext = &get_msg_plaintext(STARGAZE_WALLET_01.to_string());
    let (_, _, _, eth_addr_str) = get_wallet_and_sig(claim_plaintext.clone());

    let mut app = custom_mock_app();
    configure_mock_minter_with_mock_whitelist(&mut app);
    let minter_addr = Addr::unchecked(MOCK_MINTER_ADDR_STR);
    let airdrop_contract = Addr::unchecked(MOCK_AIRDROP_ADDR_STR);

    let params = InstantiateParams {
        addresses: vec![eth_addr_str],
        funds_amount: WHITELIST_AMOUNT + INSTANTIATION_FEE,
        expected_airdrop_contract_id: 4,
        minter_address: minter_addr.clone(),
        admin_account: Addr::unchecked(OWNER),
        app: &mut app,
        per_address_limit: 1,
        claim_msg_plaintext: CONFIG_PLAINTEXT.to_string(),
    };
    instantiate_contract(params).unwrap();
    query_minter_as_expected(&mut app, airdrop_contract.clone(), minter_addr);
    let stargaze_wallet_02 = Addr::unchecked(STARGAZE_WALLET_02);
    let balances = app
        .wrap()
        .query_all_balances(stargaze_wallet_02.clone())
        .unwrap();
    assert_eq!(balances, []);

    let claim_plaintext_2 = &get_msg_plaintext(STARGAZE_WALLET_02.to_string());
    let (_, eth_sig_str_2, _, eth_addr_str_2) = get_wallet_and_sig(claim_plaintext_2.clone());

    let claim_message = ExecuteMsg::ClaimAirdrop {
        eth_address: eth_addr_str_2.clone(),
        eth_sig: eth_sig_str_2,
    };
    let res = execute_contract_error_with_msg(
        claim_message,
        &mut app,
        stargaze_wallet_02.clone(),
        airdrop_contract,
    );
    let expected_error = format!("Address {eth_addr_str_2} is not eligible");
    assert_eq!(res, expected_error);
    let balances = app.wrap().query_all_balances(stargaze_wallet_02).unwrap();
    let expected_balance = [];
    assert_eq!(balances, expected_balance)
}

#[test]
fn test_one_eth_claim_two_stargaze_addresses_invalid() {
    let wallet_1 = LocalWallet::new(&mut thread_rng());
    let stargaze_wallet_01 = Addr::unchecked(STARGAZE_WALLET_01);
    let stargaze_wallet_02 = Addr::unchecked(STARGAZE_WALLET_02);

    let claim_plaintext_1 = &get_msg_plaintext(STARGAZE_WALLET_01.to_string());
    let eth_sig_str_1 = task::block_on(get_signature(wallet_1.clone(), claim_plaintext_1))
        .unwrap()
        .to_string();
    let eth_address = wallet_1.address();
    let eth_addr_str_1 = format!("{eth_address:?}");

    let mut app = custom_mock_app();
    configure_mock_minter_with_mock_whitelist(&mut app);
    let minter_addr = Addr::unchecked(MOCK_MINTER_ADDR_STR);
    let airdrop_contract = Addr::unchecked(MOCK_AIRDROP_ADDR_STR);

    let params = InstantiateParams {
        addresses: vec![eth_addr_str_1.clone()],
        funds_amount: WHITELIST_AMOUNT + INSTANTIATION_FEE,
        expected_airdrop_contract_id: 4,
        minter_address: minter_addr,
        admin_account: Addr::unchecked(OWNER),
        app: &mut app,
        per_address_limit: 1,
        claim_msg_plaintext: CONFIG_PLAINTEXT.to_string(),
    };
    instantiate_contract(params).unwrap();

    // claim with eth address 1, stargaze wallet 1
    let claim_message = ExecuteMsg::ClaimAirdrop {
        eth_address: eth_addr_str_1.clone(),
        eth_sig: eth_sig_str_1,
    };
    let res = execute_contract_with_msg(
        claim_message,
        &mut app,
        stargaze_wallet_01,
        airdrop_contract.clone(),
    )
    .unwrap();

    let expected_attributes = [
        Attribute {
            key: "_contract_address".to_string(),
            value: airdrop_contract.to_string(),
        },
        Attribute {
            key: "claimed_amount".to_string(),
            value: WHITELIST_AMOUNT.to_string(),
        },
    ];
    assert_eq!(res.events[1].attributes, expected_attributes);

    let claim_plaintext_2 = &get_msg_plaintext(STARGAZE_WALLET_02.to_string());
    let eth_sig_str_2 = task::block_on(get_signature(wallet_1, claim_plaintext_2))
        .unwrap()
        .to_string();

    // claim with eth address 1, stargaze wallet 2
    let claim_message = ExecuteMsg::ClaimAirdrop {
        eth_address: eth_addr_str_1.clone(),
        eth_sig: eth_sig_str_2,
    };
    let expected_error =
        format!("Address {eth_addr_str_1} has already claimed all available mints");
    let res_2 = execute_contract_error_with_msg(
        claim_message,
        &mut app,
        stargaze_wallet_02,
        airdrop_contract,
    );
    assert_eq!(res_2, expected_error);
}

#[test]
fn test_two_claims_allowed_success() {
    let claim_plaintext = &get_msg_plaintext(STARGAZE_WALLET_01.to_string());
    let (_, eth_sig_str, _, eth_addr_str) = get_wallet_and_sig(claim_plaintext.clone());

    let stargaze_wallet_01 = Addr::unchecked(STARGAZE_WALLET_01);

    let mut app = custom_mock_app();
    configure_mock_minter_with_mock_whitelist(&mut app);
    let minter_addr = Addr::unchecked(MOCK_MINTER_ADDR_STR);
    let airdrop_contract = Addr::unchecked(MOCK_AIRDROP_ADDR_STR);

    let params = InstantiateParams {
        addresses: vec![eth_addr_str.clone()],
        funds_amount: WHITELIST_AMOUNT * 2 + INSTANTIATION_FEE,
        expected_airdrop_contract_id: 4,
        minter_address: minter_addr,
        admin_account: Addr::unchecked(OWNER),
        app: &mut app,
        per_address_limit: 2,
        claim_msg_plaintext: CONFIG_PLAINTEXT.to_string(),
    };
    instantiate_contract(params).unwrap();

    let balances = app
        .wrap()
        .query_all_balances(stargaze_wallet_01.clone())
        .unwrap();
    assert_eq!(balances, []);

    let claim_message = ExecuteMsg::ClaimAirdrop {
        eth_address: eth_addr_str.clone(),
        eth_sig: eth_sig_str.clone(),
    };
    let _ = execute_contract_with_msg(
        claim_message,
        &mut app,
        stargaze_wallet_01.clone(),
        airdrop_contract.clone(),
    )
    .unwrap();

    let balances = app
        .wrap()
        .query_all_balances(stargaze_wallet_01.clone())
        .unwrap();
    let expected_balance = [Coin {
        denom: NATIVE_DENOM.to_string(),
        amount: Uint128::new(WHITELIST_AMOUNT),
    }];
    assert_eq!(balances, expected_balance);

    let claim_message = ExecuteMsg::ClaimAirdrop {
        eth_address: eth_addr_str,
        eth_sig: eth_sig_str,
    };
    let _ = execute_contract_with_msg(
        claim_message,
        &mut app,
        stargaze_wallet_01.clone(),
        airdrop_contract,
    )
    .unwrap();

    let balances = app.wrap().query_all_balances(stargaze_wallet_01).unwrap();
    let expected_balance = [Coin {
        denom: NATIVE_DENOM.to_string(),
        amount: Uint128::new(2 * WHITELIST_AMOUNT),
    }];
    assert_eq!(balances, expected_balance)
}
