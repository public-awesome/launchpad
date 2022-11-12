use crate::msg::QueryMsg;
use crate::tests_folder::claim_constants::{AIRDROP_CONTRACT, STARGAZE_WALLET_01};
use crate::tests_folder::setup_contracts::instantiate_contract;
use crate::tests_folder::setup_minter::configure_minter_with_whitelist;
use crate::tests_folder::setup_signatures::{get_msg_plaintext, get_wallet_and_sig};
use cosmwasm_std::Addr;

use super::setup_contracts::{custom_mock_app, execute_contract_with_msg};

extern crate whitelist_generic;

#[test]
fn test_set_minter_contract() {
    let mut app = custom_mock_app();
    let (minter_addr, _, _, _, config) = configure_minter_with_whitelist(&mut app);

    let claim_plaintext = &get_msg_plaintext(STARGAZE_WALLET_01.to_string());
    let (_, _, _, eth_addr_str) = get_wallet_and_sig(claim_plaintext.clone());

    let first_minter = Addr::unchecked("first_minter");
    let contract_admin = Addr::unchecked(config.admin);
    instantiate_contract(
        vec![eth_addr_str],
        10000,
        5,
        first_minter.clone(),
        contract_admin.clone(),
        &mut app,
    );
    let airdrop_contract = Addr::unchecked(AIRDROP_CONTRACT);
    let query_msg = QueryMsg::GetMinter {};
    let result: Addr = app
        .wrap()
        .query_wasm_smart(airdrop_contract.clone(), &query_msg)
        .unwrap();
    assert_eq!(result, first_minter);

    let update_minter_message = crate::msg::ExecuteMsg::UpdateMinterAddress {
        minter_address: minter_addr.to_string(),
    };

    let _ = execute_contract_with_msg(
        update_minter_message,
        &mut app,
        contract_admin,
        airdrop_contract.clone(),
    );
    let result: Addr = app
        .wrap()
        .query_wasm_smart(airdrop_contract, &query_msg)
        .unwrap();
    assert_eq!(result, minter_addr);
}

#[test]
fn test_claim_added_to_minter_whitelist() {
    let mut app = custom_mock_app();
    let (minter_addr, _, creator, _, _) = configure_minter_with_whitelist(&mut app);
    // println!("minter_config is {:?}", minter_config);

    let claim_plaintext = &get_msg_plaintext(STARGAZE_WALLET_01.to_string());
    let (_, eth_sig_str, _, eth_addr_str) = get_wallet_and_sig(claim_plaintext.clone());

    let airdrop_contract = Addr::unchecked(AIRDROP_CONTRACT);
    // let contract_admin = Addr::unchecked(minter_config.admin);
    instantiate_contract(
        vec![eth_addr_str.clone()],
        10000,
        5,
        minter_addr,
        creator.clone(),
        &mut app,
    );
    // let stargaze_wallet_01 = Addr::unchecked(STARGAZE_WALLET_01);
    let claim_message = crate::msg::ExecuteMsg::ClaimAirdrop {
        eth_address: eth_addr_str,
        eth_sig: eth_sig_str,
    };

    let res =
        execute_contract_with_msg(claim_message, &mut app, creator, airdrop_contract).unwrap();
    println!("res is {:?}", res);
}
