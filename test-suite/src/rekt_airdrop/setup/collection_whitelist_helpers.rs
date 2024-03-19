use crate::rekt_airdrop::constants::claim_constants::{NATIVE_DENOM, STARGAZE_WALLET_01};
use crate::rekt_airdrop::constants::collection_constants::{MINT_PRICE, WHITELIST_AMOUNT};

use crate::rekt_airdrop::setup::execute_msg::execute_contract_with_msg;
use cosmwasm_std::{coins, Addr};
use cw_multi_test::{BankSudo, Executor, SudoMsg};
use sg_multi_test::StargazeApp;

extern crate whitelist_immutable;

pub fn update_admin_for_whitelist(
    app: &mut StargazeApp,
    sender: Addr,
    target_admin: Addr,
    target_contract: Addr,
) {
    // add airdrop contract as admin on whitelist
    let update_admin_message = sg_whitelist::msg::ExecuteMsg::UpdateAdmins {
        admins: vec![target_admin.to_string()],
    };
    let _ = app.execute_contract(sender, target_contract, &update_admin_message, &[]);
}

pub fn send_funds_to_address(app: &mut StargazeApp, target_address_str: &str, amount: u128) {
    app.sudo(SudoMsg::Bank({
        BankSudo::Mint {
            to_address: target_address_str.to_string(),
            amount: coins(amount, NATIVE_DENOM),
        }
    }))
    .map_err(|err| println!("{err:?}"))
    .ok();
}

pub fn execute_mint_fail_not_on_whitelist(app: &mut StargazeApp, minter_addr: Addr) {
    //before mintlist add, fail
    let stargaze_wallet_01 = Addr::unchecked(STARGAZE_WALLET_01);
    let mint_msg = vending_minter::msg::ExecuteMsg::Mint {};
    let res = app.execute_contract(
        stargaze_wallet_01,
        minter_addr,
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );

    let expected_error = format!("address not on whitelist: {STARGAZE_WALLET_01}");
    assert_eq!(res.unwrap_err().root_cause().to_string(), expected_error);
}

pub fn execute_airdrop_claim(
    app: &mut StargazeApp,
    eth_addr_str: String,
    eth_sig_str: String,
    target_wallet: Addr,
    airdrop_contract: Addr,
) {
    let claim_message = rekt_airdrop::msg::ExecuteMsg::ClaimAirdrop {
        eth_address: eth_addr_str,
        eth_sig: eth_sig_str,
    };
    let _ = execute_contract_with_msg(claim_message, app, target_wallet, airdrop_contract).unwrap();
}

pub fn execute_mint_success(app: &mut StargazeApp, sender: Addr, minter_addr: Addr) {
    //execute the mint
    let mint_msg = vending_minter::msg::ExecuteMsg::Mint {};
    let res = app.execute_contract(
        sender,
        minter_addr,
        &mint_msg,
        &coins(WHITELIST_AMOUNT, NATIVE_DENOM),
    );
    assert!(res.is_ok())
}
