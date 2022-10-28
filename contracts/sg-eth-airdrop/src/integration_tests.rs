use async_std::task;
use cosmwasm_std::{Addr, Attribute};
use cw_multi_test::error::Error;
use cw_multi_test::{AppResponse, Contract, ContractWrapper, Executor};
use ethers_core::k256::ecdsa::SigningKey;
use ethers_core::types::H160;

use sg_multi_test::StargazeApp;
use sg_std::StargazeMsgWrapper;
use std::str;

use crate::msg::{AirdropEligibleResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::Config;
use ethers_core::rand::thread_rng;
use ethers_signers::{LocalWallet, Signer, Wallet, WalletError};
use eyre::Result;

const OWNER: &str = "admin0001";
const AIRDROP_CONTRACT: &str = "contract0";

fn custom_mock_app() -> StargazeApp {
    StargazeApp::default()
}

pub fn contract() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

fn get_instantiate_contract(claim_plaintext: &str) -> StargazeApp {
    let mut app = custom_mock_app();
    let true_admin = Addr::unchecked("true_admin");
    let sg_eth_id = app.store_code(contract());
    assert_eq!(sg_eth_id, 1);
    let msg: InstantiateMsg = InstantiateMsg {
        config: Config {
            admin: true_admin,
            claim_msg_plaintext: claim_plaintext.to_string(),
        },
    };
    app.instantiate_contract(
        sg_eth_id,
        Addr::unchecked(OWNER),
        &msg,
        &[],
        "sg-eg-airdrop",
        Some(Addr::unchecked(OWNER).to_string()),
    )
    .unwrap();
    app
}

async fn get_signature(
    wallet: Wallet<SigningKey>,
    plaintext_msg: &str,
) -> Result<ethers_core::types::Signature, WalletError> {
    wallet.sign_message(plaintext_msg).await
}

fn get_wallet_and_sig(
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

fn execute_contract_with_msg(msg: ExecuteMsg, app: &mut StargazeApp) -> Result<AppResponse, Error> {
    let true_admin = Addr::unchecked(OWNER);
    let sg_eth_addr = Addr::unchecked(AIRDROP_CONTRACT);

    let result = app
        .execute_contract(true_admin, sg_eth_addr, &msg, &[])
        .unwrap();
    Ok(result)
}

#[test]
fn test_instantiate() {
    let claim_plaintext = "I Want a Winter Pal.";
    get_instantiate_contract(claim_plaintext);
}

#[test]
fn test_not_authorized_add_eth() {
    let claim_plaintext = "I Want a Winter Pal.";
    let mut app = get_instantiate_contract(claim_plaintext);

    let fake_admin = Addr::unchecked("fake_admin");
    let sg_eth_addr = Addr::unchecked(AIRDROP_CONTRACT);
    let eth_address = Addr::unchecked("testing_addr");
    let execute_msg = ExecuteMsg::AddEligibleEth {
        eth_address: eth_address.to_string(),
    };
    let res = app.execute_contract(fake_admin, sg_eth_addr, &execute_msg, &[]);
    let error = res.unwrap_err();
    let expected_err_msg = "Unauthorized admin, sender is fake_admin";
    assert_eq!(error.root_cause().to_string(), expected_err_msg)
}

#[test]
fn test_authorized_add_eth() {
    let claim_plaintext = "I Want a Winter Pal.";
    let mut app = get_instantiate_contract(claim_plaintext);
    let sg_eth_addr = Addr::unchecked(AIRDROP_CONTRACT);

    let true_admin = Addr::unchecked(OWNER);
    let eth_address = Addr::unchecked("testing_addr");
    let execute_msg = ExecuteMsg::AddEligibleEth {
        eth_address: eth_address.to_string(),
    };
    let res = app.execute_contract(true_admin, sg_eth_addr, &execute_msg, &[]);
    res.unwrap();
}

#[test]
fn test_add_eth_and_verify() {
    let claim_plaintext = "I Want a Winter Pal.";

    let mut app = get_instantiate_contract(claim_plaintext);
    let sg_eth_addr = Addr::unchecked(AIRDROP_CONTRACT);

    let true_admin = Addr::unchecked(OWNER);
    let eth_address = Addr::unchecked("testing_addr");
    let execute_msg = ExecuteMsg::AddEligibleEth {
        eth_address: eth_address.to_string(),
    };

    // test before add:
    let query_msg = QueryMsg::AirdropEligible {
        eth_address: eth_address.clone(),
    };
    let expected_result = AirdropEligibleResponse { eligible: false };
    let result: AirdropEligibleResponse = app
        .wrap()
        .query_wasm_smart(sg_eth_addr.clone(), &query_msg)
        .unwrap();
    assert_eq!(result, expected_result);

    let _ = app.execute_contract(true_admin, sg_eth_addr.clone(), &execute_msg, &[]);

    //test after add
    let query_msg = QueryMsg::AirdropEligible {
        eth_address,
    };
    let expected_result = AirdropEligibleResponse { eligible: true };
    let result: AirdropEligibleResponse = app
        .wrap()
        .query_wasm_smart(sg_eth_addr, &query_msg)
        .unwrap();
    assert_eq!(result, expected_result);
}

#[test]
fn test_valid_eth_sig_claim() {
    let claim_plaintext: String = "I Want a Winter Pal.".to_string();
    let (_, eth_sig_str, _, eth_addr_str) = get_wallet_and_sig(claim_plaintext.clone());

    let execute_msg = ExecuteMsg::AddEligibleEth {
        eth_address: eth_addr_str.clone(),
    };

    let mut app = get_instantiate_contract(&claim_plaintext);
    let _ = execute_contract_with_msg(execute_msg, &mut app);

    let claim_message = ExecuteMsg::ClaimAirdrop {
        eth_address: eth_addr_str,
        eth_sig: eth_sig_str,
        stargaze_address: "abc123".to_string(),
        stargaze_sig: "0xabc123".to_string(),
    };
    let res = execute_contract_with_msg(claim_message, &mut app).unwrap();

    let expected_attributes = [
        Attribute {
            key: "_contract_addr".to_string(),
            value: "contract0".to_string(),
        },
        Attribute {
            key: "amount".to_string(),
            value: "30000".to_string(),
        },
        Attribute {
            key: "valid_eth_sig".to_string(),
            value: "true".to_string(),
        },
        Attribute {
            key: "valid_cosmos_sig".to_string(),
            value: "false".to_string(),
        },
        Attribute {
            key: "valid_claim".to_string(),
            value: "false".to_string(),
        },
        Attribute {
            key: "minter_page".to_string(),
            value: "http://levana_page/airdrop".to_string(),
        },
    ];
    assert_eq!(res.events[1].attributes, expected_attributes);
}

#[test]
fn test_invalid_eth_sig_claim() {
    let claim_plaintext: String = "I Want a Winter Pal.".to_string();

    let (_, _, _, eth_addr_str) = get_wallet_and_sig(claim_plaintext.clone());
    let (_, eth_sig_str_2, _, _) = get_wallet_and_sig(claim_plaintext.clone());

    let mut app = get_instantiate_contract(&claim_plaintext);
    let execute_msg = ExecuteMsg::AddEligibleEth {
        eth_address: eth_addr_str.clone(),
    };
    let _ = execute_contract_with_msg(execute_msg, &mut app);

    let claim_message = ExecuteMsg::ClaimAirdrop {
        eth_address: eth_addr_str,
        eth_sig: eth_sig_str_2,
        stargaze_address: "abc123".to_string(),
        stargaze_sig: "0xabc123".to_string(),
    };
    let res = execute_contract_with_msg(claim_message, &mut app).unwrap();

    let expected_attributes = [
        Attribute {
            key: "_contract_addr".to_string(),
            value: "contract0".to_string(),
        },
        Attribute {
            key: "amount".to_string(),
            value: "30000".to_string(),
        },
        Attribute {
            key: "valid_eth_sig".to_string(),
            value: "false".to_string(),
        },
        Attribute {
            key: "valid_cosmos_sig".to_string(),
            value: "false".to_string(),
        },
        Attribute {
            key: "valid_claim".to_string(),
            value: "false".to_string(),
        },
        Attribute {
            key: "minter_page".to_string(),
            value: "http://levana_page/airdrop".to_string(),
        },
    ];
    assert_eq!(res.events[1].attributes, expected_attributes);
}
