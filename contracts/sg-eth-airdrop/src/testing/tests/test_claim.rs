use crate::msg::ExecuteMsg;
use crate::tests_folder::collection_constants::WHITELIST_AMOUNT;
use async_std::task;
use cosmwasm_std::{Addr, Attribute, Coin, Uint128};

use ethers_core::rand::thread_rng;
use ethers_signers::{LocalWallet, Signer};

use crate::tests_folder::claim_constants::{
    MOCK_AIRDROP_ADDR_STR, MOCK_MINTER_ADDR_STR, NATIVE_DENOM, OWNER, STARGAZE_WALLET_01,
    STARGAZE_WALLET_02,
};
use crate::tests_folder::setup_contracts::{
    custom_mock_app, execute_contract_with_msg, instantiate_contract, instantiate_contract_get_app,
};
use crate::tests_folder::setup_minter::configure_mock_minter_with_mock_whitelist;
use crate::tests_folder::setup_signatures::{get_msg_plaintext, get_signature, get_wallet_and_sig};

#[test]
fn test_instantiate() {
    let minter_address = Addr::unchecked("contract1");
    instantiate_contract_get_app(vec![], 10000, 1, minter_address);
}

#[test]
fn test_valid_eth_sig_claim() {
    let claim_plaintext = &get_msg_plaintext(STARGAZE_WALLET_01.to_string());
    let (_, eth_sig_str, _, eth_addr_str) = get_wallet_and_sig(claim_plaintext.clone());
    let mut app = custom_mock_app();
    configure_mock_minter_with_mock_whitelist(&mut app);
    let minter_addr = Addr::unchecked(MOCK_MINTER_ADDR_STR);
    let airdrop_contract = Addr::unchecked(MOCK_AIRDROP_ADDR_STR);

    instantiate_contract(
        vec![eth_addr_str.clone()],
        WHITELIST_AMOUNT,
        4,
        minter_addr.clone(),
        Addr::unchecked(OWNER),
        &mut app,
    );

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
            key: "_contract_addr".to_string(),
            value: airdrop_contract.to_string(),
        },
        Attribute {
            key: "claimed_amount".to_string(),
            value: WHITELIST_AMOUNT.to_string(),
        },
        Attribute {
            key: "valid_eth_sig".to_string(),
            value: "true".to_string(),
        },
        Attribute {
            key: "eligible_at_request".to_string(),
            value: "true".to_string(),
        },
        Attribute {
            key: "minter_address".to_string(),
            value: minter_addr.to_string(),
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

    instantiate_contract(
        vec![eth_addr_str.clone()],
        WHITELIST_AMOUNT,
        4,
        minter_addr.clone(),
        Addr::unchecked(OWNER),
        &mut app,
    );

    let claim_message = ExecuteMsg::ClaimAirdrop {
        eth_address: eth_addr_str,
        eth_sig: eth_sig_str_2,
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
            key: "_contract_addr".to_string(),
            value: airdrop_contract.to_string(),
        },
        Attribute {
            key: "claimed_amount".to_string(),
            value: "0".to_string(),
        },
        Attribute {
            key: "valid_eth_sig".to_string(),
            value: "false".to_string(),
        },
        Attribute {
            key: "eligible_at_request".to_string(),
            value: "true".to_string(),
        },
        Attribute {
            key: "minter_address".to_string(),
            value: minter_addr.to_string(),
        },
    ];
    assert_eq!(res.events[1].attributes, expected_attributes);
}

#[test]
fn test_can_not_claim_twice() {
    let claim_plaintext = &get_msg_plaintext(STARGAZE_WALLET_01.to_string());
    let (_, eth_sig_str, _, eth_addr_str) = get_wallet_and_sig(claim_plaintext.clone());

    let mut app = custom_mock_app();
    configure_mock_minter_with_mock_whitelist(&mut app);
    let minter_addr = Addr::unchecked(MOCK_MINTER_ADDR_STR);
    let airdrop_contract = Addr::unchecked(MOCK_AIRDROP_ADDR_STR);

    instantiate_contract(
        vec![eth_addr_str.clone()],
        WHITELIST_AMOUNT,
        4,
        minter_addr.clone(),
        Addr::unchecked(OWNER),
        &mut app,
    );

    let claim_message = ExecuteMsg::ClaimAirdrop {
        eth_address: eth_addr_str,
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
            key: "_contract_addr".to_string(),
            value: airdrop_contract.to_string(),
        },
        Attribute {
            key: "claimed_amount".to_string(),
            value: WHITELIST_AMOUNT.to_string(),
        },
        Attribute {
            key: "valid_eth_sig".to_string(),
            value: "true".to_string(),
        },
        Attribute {
            key: "eligible_at_request".to_string(),
            value: "true".to_string(),
        },
        Attribute {
            key: "minter_address".to_string(),
            value: minter_addr.to_string(),
        },
    ];
    assert_eq!(res.events[1].attributes, expected_attributes);
    let res = execute_contract_with_msg(
        claim_message,
        &mut app,
        stargaze_wallet_01,
        airdrop_contract.clone(),
    )
    .unwrap();
    let expected_attributes = [
        Attribute {
            key: "_contract_addr".to_string(),
            value: airdrop_contract.to_string(),
        },
        Attribute {
            key: "claimed_amount".to_string(),
            value: "0".to_string(),
        },
        Attribute {
            key: "valid_eth_sig".to_string(),
            value: "true".to_string(),
        },
        Attribute {
            key: "eligible_at_request".to_string(),
            value: "false".to_string(),
        },
        Attribute {
            key: "minter_address".to_string(),
            value: minter_addr.to_string(),
        },
    ];
    assert_eq!(res.events[1].attributes, expected_attributes);
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

    instantiate_contract(
        vec![eth_addr_str.clone()],
        WHITELIST_AMOUNT,
        4,
        minter_addr,
        Addr::unchecked(OWNER),
        &mut app,
    );

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

    instantiate_contract(
        vec![eth_addr_str.clone()],
        WHITELIST_AMOUNT,
        4,
        minter_addr,
        Addr::unchecked(OWNER),
        &mut app,
    );
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

    instantiate_contract(
        vec![eth_addr_str],
        WHITELIST_AMOUNT,
        4,
        minter_addr,
        Addr::unchecked(OWNER),
        &mut app,
    );

    let stargaze_wallet_02 = Addr::unchecked(STARGAZE_WALLET_02);
    let balances = app
        .wrap()
        .query_all_balances(stargaze_wallet_02.clone())
        .unwrap();
    assert_eq!(balances, []);

    let claim_plaintext_2 = &get_msg_plaintext(STARGAZE_WALLET_02.to_string());
    let (_, eth_sig_str_2, _, eth_addr_str_2) = get_wallet_and_sig(claim_plaintext_2.clone());

    let claim_message = ExecuteMsg::ClaimAirdrop {
        eth_address: eth_addr_str_2,
        eth_sig: eth_sig_str_2,
    };
    let _ = execute_contract_with_msg(
        claim_message,
        &mut app,
        stargaze_wallet_02.clone(),
        airdrop_contract,
    )
    .unwrap();

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
    let eth_addr_str_1 = format!("{:?}", eth_address);

    let mut app = custom_mock_app();
    configure_mock_minter_with_mock_whitelist(&mut app);
    let minter_addr = Addr::unchecked(MOCK_MINTER_ADDR_STR);
    let airdrop_contract = Addr::unchecked(MOCK_AIRDROP_ADDR_STR);

    instantiate_contract(
        vec![eth_addr_str_1.clone()],
        WHITELIST_AMOUNT,
        4,
        minter_addr.clone(),
        Addr::unchecked(OWNER),
        &mut app,
    );

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
            key: "_contract_addr".to_string(),
            value: airdrop_contract.to_string(),
        },
        Attribute {
            key: "claimed_amount".to_string(),
            value: WHITELIST_AMOUNT.to_string(),
        },
        Attribute {
            key: "valid_eth_sig".to_string(),
            value: "true".to_string(),
        },
        Attribute {
            key: "eligible_at_request".to_string(),
            value: "true".to_string(),
        },
        Attribute {
            key: "minter_address".to_string(),
            value: minter_addr.to_string(),
        },
    ];
    assert_eq!(res.events[1].attributes, expected_attributes);

    let claim_plaintext_2 = &get_msg_plaintext(STARGAZE_WALLET_02.to_string());
    let eth_sig_str_2 = task::block_on(get_signature(wallet_1, claim_plaintext_2))
        .unwrap()
        .to_string();

    // claim with eth address 1, stargaze wallet 2
    let claim_message = ExecuteMsg::ClaimAirdrop {
        eth_address: eth_addr_str_1,
        eth_sig: eth_sig_str_2,
    };

    let res_2 = execute_contract_with_msg(
        claim_message,
        &mut app,
        stargaze_wallet_02,
        airdrop_contract.clone(),
    )
    .unwrap();

    let expected_attributes = [
        Attribute {
            key: "_contract_addr".to_string(),
            value: airdrop_contract.to_string(),
        },
        Attribute {
            key: "claimed_amount".to_string(),
            value: "0".to_string(),
        },
        Attribute {
            key: "valid_eth_sig".to_string(),
            value: "true".to_string(),
        },
        Attribute {
            key: "eligible_at_request".to_string(),
            value: "false".to_string(),
        },
        Attribute {
            key: "minter_address".to_string(),
            value: minter_addr.to_string(),
        },
    ];
    assert_eq!(res_2.events[1].attributes, expected_attributes);
}
