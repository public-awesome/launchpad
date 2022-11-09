use async_std::task;
use cosmwasm_std::{coins, Addr};
use cw_multi_test::error::Error;
use cw_multi_test::{AppResponse, BankSudo, Contract, ContractWrapper, Executor, SudoMsg};
use ethers_core::k256::ecdsa::SigningKey;
use ethers_core::types::H160;
use std::str;

use sg_multi_test::StargazeApp;
use sg_std::{self, StargazeMsgWrapper};

use crate::msg::{ExecuteMsg, InstantiateMsg};

use ethers_core::rand::thread_rng;
use ethers_signers::{LocalWallet, Signer, Wallet, WalletError};
use eyre::Result;

extern crate whitelist_generic;
use crate::tests_folder::constants::{
    AIRDROP_CONTRACT, CONTRACT_CONFIG_PLAINTEXT, NATIVE_DENOM, OWNER,
};

pub fn custom_mock_app() -> StargazeApp {
    StargazeApp::default()
}

pub fn contract() -> Box<dyn Contract<sg_std::StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    )
    .with_reply(crate::contract::reply);
    Box::new(contract)
}

pub fn whitelist_generic_contract() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        whitelist_generic::contract::execute,
        whitelist_generic::contract::instantiate,
        whitelist_generic::contract::query,
    );
    Box::new(contract)
}

pub fn get_instantiate_contract(addresses: Vec<String>, funds_amount: u128) -> StargazeApp {
    let mut app = custom_mock_app();
    app.sudo(SudoMsg::Bank({
        BankSudo::Mint {
            to_address: OWNER.to_string(),
            amount: coins(funds_amount, NATIVE_DENOM),
        }
    }))
    .map_err(|err| println!("{:?}", err))
    .ok();

    let sg_eth_id = app.store_code(contract());
    let whitelist_code_id = app.store_code(whitelist_generic_contract());
    assert_eq!(sg_eth_id, 1);
    let msg: InstantiateMsg = InstantiateMsg {
        admin: Addr::unchecked(OWNER),
        claim_msg_plaintext: Addr::unchecked(CONTRACT_CONFIG_PLAINTEXT).into_string(),
        airdrop_amount: 3000,
        minter_page: "http://levana_page/airdrop".to_string(),
        addresses,
        minter_code_id: whitelist_code_id,
    };
    app.instantiate_contract(
        sg_eth_id,
        Addr::unchecked(OWNER),
        &msg,
        &coins(funds_amount, NATIVE_DENOM),
        "sg-eg-airdrop",
        Some(Addr::unchecked(OWNER).to_string()),
    )
    .unwrap();
    app
}

pub async fn get_signature(
    wallet: Wallet<SigningKey>,
    plaintext_msg: &str,
) -> Result<ethers_core::types::Signature, WalletError> {
    wallet.sign_message(plaintext_msg).await
}

pub fn get_wallet_and_sig(
    claim_plaintext: String,
) -> (
    Wallet<ethers_core::k256::ecdsa::SigningKey>,
    std::string::String,
    H160,
    std::string::String,
) {
    let wallet = LocalWallet::new(&mut thread_rng());
    let eth_sig_str = task::block_on(get_signature(wallet.clone(), &claim_plaintext))
        .unwrap()
        .to_string();
    let eth_address = wallet.address();
    let eth_addr_str = format!("{:?}", eth_address);
    (wallet, eth_sig_str, eth_address, eth_addr_str)
}

pub fn execute_contract_with_msg(
    msg: ExecuteMsg,
    app: &mut StargazeApp,
    user: Addr,
) -> Result<AppResponse, Error> {
    let sg_eth_addr = Addr::unchecked(AIRDROP_CONTRACT);

    let result = app.execute_contract(user, sg_eth_addr, &msg, &[]).unwrap();
    Ok(result)
}

pub fn get_msg_plaintext(wallet_address: String) -> String {
    str::replace(CONTRACT_CONFIG_PLAINTEXT, "{wallet}", &wallet_address)
}
