use cosmwasm_std::{coin, coins, Addr, Timestamp};
use cw_multi_test::{BankSudo, Contract, ContractWrapper, Executor, SudoMsg};

use sg2::msg::Sg2ExecuteMsg;
use sg_multi_test::StargazeApp;
use sg_std::{self, StargazeMsgWrapper, GENESIS_MINT_START_TIME, NATIVE_DENOM};

use crate::contract::reply;
use crate::msg::QueryMsg;
use crate::tests_folder::constants::STARGAZE_WALLET_01;
use crate::tests_folder::shared::{
    get_msg_plaintext, get_wallet_and_sig, instantiate_contract, instantiate_contract_get_app,
};
use sg2::tests::mock_collection_params;
use vending_factory::msg::{VendingMinterCreateMsg, VendingMinterInitMsgExtension};
use vending_factory::state::{ParamsExtension, VendingMinterParams};

use super::constants::OWNER;
use super::shared::{custom_mock_app, execute_contract_with_msg};

extern crate whitelist_generic;

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

pub fn contract() -> Box<dyn Contract<sg_std::StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    )
    .with_reply(reply);
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
        .query_wasm_smart(
            minter_addr.clone(),
            &vending_minter::msg::QueryMsg::Config {},
        )
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

fn configure_collection_whitelist(
    router: &mut StargazeApp,
    creator: Addr,
    buyer: Addr,
    minter_addr: Addr,
) -> Addr {
    let whitelist_addr = setup_whitelist_contract(router, &creator);
    const AFTER_GENESIS_TIME: Timestamp = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 100);

    // Set to just before genesis mint start time
    setup_block_time(router, GENESIS_MINT_START_TIME - 10, None);

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
    assert!(res.is_ok());
    whitelist_addr
}

fn configure_minter_with_whitelist(app: &mut StargazeApp) -> (Addr, Addr, Addr, Addr) {
    let (creator, buyer) = setup_accounts(app);
    let num_tokens = 1;
    let (minter_addr, config) = setup_minter_contract(app, &creator, num_tokens, None);
    let sg721_addr = config.sg721_address;

    let whitelist_addr =
        configure_collection_whitelist(app, creator.clone(), buyer.clone(), minter_addr.clone());

    setup_block_time(app, GENESIS_MINT_START_TIME, None);
    (minter_addr, whitelist_addr, creator, buyer)
}

#[test]
fn test_set_minter_contract() {
    let mut app = custom_mock_app();
    let (minter_addr, _, _, _) = configure_minter_with_whitelist(&mut app);

    let claim_plaintext = &get_msg_plaintext(STARGAZE_WALLET_01.to_string());
    let (_, _, _, eth_addr_str) = get_wallet_and_sig(claim_plaintext.clone());

    let first_minter = Addr::unchecked("first_minter");
    instantiate_contract(
        vec![eth_addr_str.clone()],
        10000,
        5,
        first_minter.clone(),
        &mut app,
    );
    let airdrop_contract = Addr::unchecked("contract4");
    let query_msg = QueryMsg::GetMinter {};
    let result: Addr = app
        .wrap()
        .query_wasm_smart(airdrop_contract.clone(), &query_msg)
        .unwrap();
    assert_eq!(result, first_minter);

    let owner_admin = Addr::unchecked(OWNER);
    let update_minter_message = crate::msg::ExecuteMsg::UpdateMinterAddress {
        minter_address: minter_addr.to_string(),
    };

    let _ = execute_contract_with_msg(
        update_minter_message,
        &mut app,
        owner_admin,
        airdrop_contract.clone(),
    );
    let result: Addr = app
        .wrap()
        .query_wasm_smart(airdrop_contract.clone(), &query_msg)
        .unwrap();
    assert_eq!(result, minter_addr);
}
