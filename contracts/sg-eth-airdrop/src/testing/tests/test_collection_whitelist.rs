use cosmwasm_std::Addr;

use crate::msg::QueryMsg;
use crate::tests_folder::constants::{
    AIRDROP_ADDR_STR, MINT_PRICE, STARGAZE_WALLET_01, WHITELIST_AMOUNT,
};
use crate::tests_folder::tests_setup::{
    configure_minter_with_whitelist, custom_mock_app, execute_airdrop_claim,
    execute_mint_fail_not_on_whitelist, execute_mint_success, get_msg_plaintext,
    get_wallet_and_sig, instantiate_contract, send_funds_to_address, update_admin_for_whitelist,
    InstantiateParams,
};
extern crate whitelist_immutable;
use crate::helpers::INSTANTIATION_FEE;

#[test]
fn test_set_minter_contract_success() {
    let mut app = custom_mock_app();
    let (minter_addr, _, _, _, config) = configure_minter_with_whitelist(&mut app);

    let claim_plaintext = &get_msg_plaintext(STARGAZE_WALLET_01.to_string());
    let (_, _, _, eth_addr_str) = get_wallet_and_sig(claim_plaintext.clone());

    let contract_admin = Addr::unchecked(config.admin);
    let params = InstantiateParams {
        addresses: vec![eth_addr_str],
        funds_amount: WHITELIST_AMOUNT + INSTANTIATION_FEE,
        expected_airdrop_contract_id: 5,
        minter_address: minter_addr.clone(),
        admin_account: contract_admin,
        app: &mut app,
        per_address_limit: 1,
    };
    instantiate_contract(params);
    let airdrop_contract = Addr::unchecked(AIRDROP_ADDR_STR);
    let query_msg = QueryMsg::GetMinter {};
    let result: Addr = app
        .wrap()
        .query_wasm_smart(airdrop_contract, &query_msg)
        .unwrap();
    assert_eq!(result, minter_addr);
}

#[test]
fn test_claim_added_to_minter_whitelist() {
    let mut app = custom_mock_app();
    let (minter_addr, whiltelist_addr, creator, _, _) = configure_minter_with_whitelist(&mut app);
    let claim_plaintext = &get_msg_plaintext(STARGAZE_WALLET_01.to_string());
    let (_, eth_sig_str, _, eth_addr_str) = get_wallet_and_sig(claim_plaintext.clone());

    let airdrop_contract = Addr::unchecked(AIRDROP_ADDR_STR);
    let params = InstantiateParams {
        addresses: vec![eth_addr_str.clone()],
        funds_amount: WHITELIST_AMOUNT + INSTANTIATION_FEE,
        expected_airdrop_contract_id: 5,
        minter_address: minter_addr.clone(),
        admin_account: creator.clone(),
        app: &mut app,
        per_address_limit: 1,
    };
    instantiate_contract(params);

    let stargaze_wallet_01 = Addr::unchecked(STARGAZE_WALLET_01);
    update_admin_for_whitelist(&mut app, creator, airdrop_contract.clone(), whiltelist_addr);
    send_funds_to_address(&mut app, STARGAZE_WALLET_01, MINT_PRICE);
    execute_mint_fail_not_on_whitelist(&mut app, minter_addr.clone());
    execute_airdrop_claim(
        &mut app,
        eth_addr_str,
        eth_sig_str,
        stargaze_wallet_01.clone(),
        airdrop_contract,
    );
    execute_mint_success(&mut app, stargaze_wallet_01, minter_addr);
}
