use async_std::task;
use cosmwasm_std::{Addr, Attribute, Timestamp, coin, coins};
use cw_multi_test::error::Error;
use cw_multi_test::{AppResponse, Contract, ContractWrapper, Executor, SudoMsg, BankSudo};
use ethers_core::k256::ecdsa::SigningKey;
use ethers_core::types::H160;

use sg2::msg::Sg2ExecuteMsg;
use sg_multi_test::StargazeApp;
use sg_std::{self, StargazeMsgWrapper, GENESIS_MINT_START_TIME, NATIVE_DENOM};

// use sg_std::StargazeMsgWrapper;
use std::str;

use crate::contract::reply;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

use ethers_core::rand::thread_rng;
use ethers_signers::{LocalWallet, Signer, Wallet, WalletError};
use eyre::Result;
use sg2::tests::mock_collection_params;
use vending_factory::msg::{VendingMinterCreateMsg, VendingMinterInitMsgExtension};
use vending_factory::state::{ParamsExtension, VendingMinterParams};

extern crate whitelist_generic;

const OWNER: &str = "admin0001";
const AIRDROP_CONTRACT: &str = "contract0";
const STARGAZE_WALLET_01: &str = "0xstargaze_wallet_01";
const CONTRACT_CONFIG_PLAINTEXT: &str = "My Stargaze address is {wallet} and I want a Winter Pal.";

const CREATION_FEE: u128 = 5_000_000_000;
const INITIAL_BALANCE: u128 = 2_000_000_000;

const MINT_PRICE: u128 = 100_000_000;
const MINT_FEE: u128 = 10_000_000;
const WHITELIST_AMOUNT: u128 = 66_000_000;
const WL_PER_ADDRESS_LIMIT: u32 = 1;
const ADMIN_MINT_PRICE: u128 = 0;
const MAX_TOKEN_LIMIT: u32 = 10000;

pub const MIN_MINT_PRICE: u128 = 50_000_000;
pub const AIRDROP_MINT_PRICE: u128 = 0;
pub const MINT_FEE_BPS: u64 = 1_000; // 10%
pub const AIRDROP_MINT_FEE_BPS: u64 = 10_000; // 100%
pub const SHUFFLE_FEE: u128 = 500_000_000;
pub const MAX_PER_ADDRESS_LIMIT: u32 = 50;

fn custom_mock_app() -> StargazeApp {
    StargazeApp::default()
}

// pub fn contract() -> Box<dyn Contract<sg_std::StargazeMsgWrapper>> {
//     let contract = ContractWrapper::new(
//         crate::contract::execute,
//         crate::contract::instantiate,
//         crate::contract::query,
//     )
//     .with_reply(reply);
//     Box::new(contract)
// }

pub fn whitelist_generic_contract() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        whitelist_generic::contract::execute,
        whitelist_generic::contract::instantiate,
        whitelist_generic::contract::query,
    );
    Box::new(contract)
}

pub fn contract_minter() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        vending_minter::contract::execute,
        vending_minter::contract::instantiate,
        vending_minter::contract::query,
    )
    .with_reply(vending_minter::contract::reply);
    Box::new(contract)
}


pub fn contract_whitelist() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        sg_whitelist::contract::execute,
        sg_whitelist::contract::instantiate,
        sg_whitelist::contract::query,
    );
    Box::new(contract)
}

pub fn contract_factory() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        vending_factory::contract::execute,
        vending_factory::contract::instantiate,
        vending_factory::contract::query,
    );
    Box::new(contract)
}

pub fn contract_sg721() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        sg721_base::entry::execute,
        sg721_base::entry::instantiate,
        sg721_base::entry::query,
    );
    Box::new(contract)
}



fn setup_whitelist_contract(router: &mut StargazeApp, creator: &Addr) -> Addr {
    let whitelist_code_id = router.store_code(contract_whitelist());

    let msg = sg_whitelist::msg::InstantiateMsg {
        members: vec![],
        start_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME + 100),
        end_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10000000),
        mint_price: coin(WHITELIST_AMOUNT, NATIVE_DENOM),
        per_address_limit: WL_PER_ADDRESS_LIMIT,
        member_limit: 1000,
    };
    router
        .instantiate_contract(
            whitelist_code_id,
            creator.clone(),
            &msg,
            &[coin(100_000_000, NATIVE_DENOM)],
            "whitelist",
            None,
        )
        .unwrap()
}


// fn get_instantiate_contract(addresses: Vec<String>, funds_amount: u128) -> StargazeApp {
//     let mut app = custom_mock_app();
//     app.sudo(SudoMsg::Bank({
//         BankSudo::Mint {
//             to_address: OWNER.to_string(),
//             amount: coins(funds_amount, NATIVE_DENOM),
//         }
//     }))
//     .map_err(|err| println!("{:?}", err))
//     .ok();

//     let sg_eth_id = app.store_code(contract());
//     let whitelist_code_id = app.store_code(whitelist_generic_contract());
//     assert_eq!(sg_eth_id, 1);
//     let msg: InstantiateMsg = InstantiateMsg {
//         admin: Addr::unchecked(OWNER),
//         claim_msg_plaintext: Addr::unchecked(CONTRACT_CONFIG_PLAINTEXT).into_string(),
//         airdrop_amount: 3000,
//         minter_page: "http://levana_page/airdrop".to_string(),
//         addresses,
//         minter_code_id: whitelist_code_id,
//     };
//     app.instantiate_contract(
//         sg_eth_id,
//         Addr::unchecked(OWNER),
//         &msg,
//         &coins(funds_amount, NATIVE_DENOM),
//         "sg-eg-airdrop",
//         Some(Addr::unchecked(OWNER).to_string()),
//     )
//     .unwrap();
//     app
// }


pub fn mock_init_extension(splits_addr: Option<String>) -> VendingMinterInitMsgExtension {
    vending_factory::msg::VendingMinterInitMsgExtension {
        base_token_uri: "ipfs://aldkfjads".to_string(),
        payment_address: splits_addr,
        start_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME),
        num_tokens: 100,
        mint_price: coin(MIN_MINT_PRICE, NATIVE_DENOM),
        per_address_limit: 5,
        whitelist: None,
    }
}

pub fn mock_create_minter(splits_addr: Option<String>) -> VendingMinterCreateMsg {
    VendingMinterCreateMsg {
        init_msg: mock_init_extension(splits_addr),
        collection_params: mock_collection_params(),
    }
}

pub fn mock_params() -> VendingMinterParams {
    VendingMinterParams {
        code_id: 1,
        creation_fee: coin(CREATION_FEE, NATIVE_DENOM),
        min_mint_price: coin(MIN_MINT_PRICE, NATIVE_DENOM),
        mint_fee_bps: MINT_FEE_BPS,
        max_trading_offset_secs: 60 * 60 * 24 * 7,
        extension: ParamsExtension {
            max_token_limit: MAX_TOKEN_LIMIT,
            max_per_address_limit: MAX_PER_ADDRESS_LIMIT,
            airdrop_mint_price: coin(AIRDROP_MINT_PRICE, NATIVE_DENOM),
            airdrop_mint_fee_bps: AIRDROP_MINT_FEE_BPS,
            shuffle_fee: coin(SHUFFLE_FEE, NATIVE_DENOM),
        },
    }
}


// Upload contract code and instantiate minter contract
fn setup_minter_contract(
    router: &mut StargazeApp,
    creator: &Addr,
    num_tokens: u32,
    splits_addr: Option<String>,
) -> (Addr, vending_minter::msg::ConfigResponse) {
    let minter_code_id = router.store_code(contract_minter());
    println!("minter_code_id: {}", minter_code_id);
    let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);

    let factory_code_id = router.store_code(contract_factory());
    println!("factory_code_id: {}", factory_code_id);

    let mut params = mock_params();
    params.code_id = minter_code_id;

    let factory_addr = router
        .instantiate_contract(
            factory_code_id,
            creator.clone(),
            &vending_factory::msg::InstantiateMsg { params },
            &[],
            "factory",
            None,
        )
        .unwrap();
    
    let sg721_code_id = router.store_code(contract_sg721());
    println!("sg721_code_id: {}", sg721_code_id);

    let mut msg = mock_create_minter(splits_addr);
    msg.init_msg.mint_price = coin(MINT_PRICE, NATIVE_DENOM);
    msg.init_msg.num_tokens = num_tokens;
    msg.collection_params.code_id = sg721_code_id;
    msg.collection_params.info.creator = creator.to_string();

    let msg = Sg2ExecuteMsg::CreateMinter(msg);

    let res = router.execute_contract(creator.clone(), factory_addr, &msg, &creation_fee);
    assert!(res.is_ok());

    // could get the minter address from the response above, but we know its contract1
    let minter_addr = Addr::unchecked("contract1");

    let config: vending_minter::msg::ConfigResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &vending_minter::msg::QueryMsg::Config {})
        .unwrap();

    (minter_addr, config)
}

// Add a creator account with initial balances
fn setup_accounts(router: &mut StargazeApp) -> (Addr, Addr) {
    let buyer = Addr::unchecked("buyer");
    let creator = Addr::unchecked("creator");
    // 3,000 tokens
    let creator_funds = coins(INITIAL_BALANCE + CREATION_FEE, NATIVE_DENOM);
    // 2,000 tokens
    let buyer_funds = coins(INITIAL_BALANCE, NATIVE_DENOM);
    router
        .sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: creator.to_string(),
                amount: creator_funds.clone(),
            }
        }))
        .map_err(|err| println!("{:?}", err))
        .ok();

    router
        .sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: buyer.to_string(),
                amount: buyer_funds.clone(),
            }
        }))
        .map_err(|err| println!("{:?}", err))
        .ok();

    // Check native balances
    let creator_native_balances = router.wrap().query_all_balances(creator.clone()).unwrap();
    assert_eq!(creator_native_balances, creator_funds);

    // Check native balances
    let buyer_native_balances = router.wrap().query_all_balances(buyer.clone()).unwrap();
    assert_eq!(buyer_native_balances, buyer_funds);

    (creator, buyer)
}

// Set blockchain time to after mint by default
fn setup_block_time(router: &mut StargazeApp, nanos: u64, height: Option<u64>) {
    let mut block = router.block_info();
    block.time = Timestamp::from_nanos(nanos);
    if let Some(h) = height {
        block.height = h;
    }
    router.set_block(block);
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

fn execute_contract_with_msg(
    msg: ExecuteMsg,
    app: &mut StargazeApp,
    user: Addr,
) -> Result<AppResponse, Error> {
    let sg_eth_addr = Addr::unchecked(AIRDROP_CONTRACT);

    let result = app.execute_contract(user, sg_eth_addr, &msg, &[]).unwrap();
    Ok(result)
}

fn get_msg_plaintext(wallet_address: String) -> String {
    str::replace(CONTRACT_CONFIG_PLAINTEXT, "{wallet}", &wallet_address)
}

// #[test]
// fn test_instantiate() {
//     get_instantiate_contract(vec![], 10000);
// }

#[test]
fn whitelist_access_len_add_remove_expiration() {
    let mut router = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut router);
    let num_tokens = 1;
    let (minter_addr, config) = setup_minter_contract(&mut router, &creator, num_tokens, None);
    let sg721_addr = config.sg721_address;
    let whitelist_addr = setup_whitelist_contract(&mut router, &creator);
    const AFTER_GENESIS_TIME: Timestamp = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 100);

    // Set to just before genesis mint start time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME - 10, None);

    // Update whitelist_expiration fails if not admin
    let wl_msg = sg_whitelist::msg::ExecuteMsg::UpdateEndTime(AFTER_GENESIS_TIME);
    router
        .execute_contract(buyer.clone(), whitelist_addr.clone(), &wl_msg, &[])
        .unwrap_err();

    // Update whitelist_expiration succeeds when from admin
    let wl_msg = sg_whitelist::msg::ExecuteMsg::UpdateEndTime(AFTER_GENESIS_TIME);
    let res = router.execute_contract(creator.clone(), whitelist_addr.clone(), &wl_msg, &[]);
    assert!(res.is_ok());

    let wl_msg = sg_whitelist::msg::ExecuteMsg::UpdateStartTime(Timestamp::from_nanos(0));
    let res = router.execute_contract(creator.clone(), whitelist_addr.clone(), &wl_msg, &[]);
    assert!(res.is_ok());

    // Set whitelist in minter contract
    let set_whitelist_msg = vending_minter::msg::ExecuteMsg::SetWhitelist {
        whitelist: whitelist_addr.to_string(),
    };
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &set_whitelist_msg,
        &[],
    );
    // println!("response is {:?}", res.unwrap());
    assert!(res.is_ok());
}