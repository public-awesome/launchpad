use cosmwasm_std::{coin, coins, Addr, Timestamp};
use cw_multi_test::{Contract, ContractWrapper, Executor};
use sg2::{msg::Sg2ExecuteMsg, tests::mock_collection_params};
use sg_multi_test::StargazeApp;
use sg_std::{StargazeMsgWrapper, GENESIS_MINT_START_TIME, NATIVE_DENOM};
use sg_whitelist::msg::InstantiateMsg as WhitelistInstantiateMsg;
use vending_factory::{
    msg::{VendingMinterCreateMsg, VendingMinterInitMsgExtension},
    state::{ParamsExtension, VendingMinterParams},
};

use crate::msg::ConfigResponse;

const CREATION_FEE: u128 = 5_000_000_000;

const MINT_PRICE: u128 = 100_000_000;
const WHITELIST_AMOUNT: u128 = 66_000_000;
const WL_PER_ADDRESS_LIMIT: u32 = 1;
const MAX_TOKEN_LIMIT: u32 = 10000;

pub const MIN_MINT_PRICE: u128 = 50_000_000;
pub const AIRDROP_MINT_PRICE: u128 = 0;
pub const MINT_FEE_BPS: u64 = 1_000; // 10%
pub const AIRDROP_MINT_FEE_BPS: u64 = 10_000; // 100%
pub const SHUFFLE_FEE: u128 = 500_000_000;
pub const MAX_PER_ADDRESS_LIMIT: u32 = 50;

pub fn custom_mock_app() -> StargazeApp {
    StargazeApp::default()
}

pub fn contract_factory() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        vending_factory::contract::execute,
        vending_factory::contract::instantiate,
        vending_factory::contract::query,
    );
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

pub fn contract_minter() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    )
    .with_reply(crate::contract::reply);
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

pub fn contract_splits() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new_with_empty(
        sg_splits::contract::execute,
        sg_splits::contract::instantiate,
        sg_splits::contract::query,
    );
    Box::new(contract)
}

pub fn contract_group() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new_with_empty(
        cw4_group::contract::execute,
        cw4_group::contract::instantiate,
        cw4_group::contract::query,
    );
    Box::new(contract)
}

pub fn setup_whitelist_contract(router: &mut StargazeApp, creator: &Addr) -> Addr {
    let whitelist_code_id = router.store_code(contract_whitelist());

    let msg = WhitelistInstantiateMsg {
        members: vec![],
        start_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME + 100),
        end_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10000000),
        mint_price: coin(WHITELIST_AMOUNT, NATIVE_DENOM),
        per_address_limit: WL_PER_ADDRESS_LIMIT,
        member_limit: 1000,
        admins: vec![creator.to_string()],
        admins_mutable: true,
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

// Upload contract code and instantiate minter contract
pub fn setup_minter_contract(
    router: &mut StargazeApp,
    creator: &Addr,
    num_tokens: u32,
    splits_addr: Option<String>,
) -> (Addr, ConfigResponse) {
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

    let config: ConfigResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &crate::msg::QueryMsg::Config {})
        .unwrap();

    (minter_addr, config)
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

pub fn mock_init_extension(splits_addr: Option<String>) -> VendingMinterInitMsgExtension {
    VendingMinterInitMsgExtension {
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

pub fn setup_minter_contract_with_splits(
    router: &mut StargazeApp,
    creator: &Addr,
    num_tokens: u32,
    splits_addr: Option<String>,
) -> (Addr, ConfigResponse) {
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

    // 1 = group, 2 = splits, 3 = minter, 4 = factory, 5 = sg721
    let minter_addr = Addr::unchecked("contract3");

    let config: ConfigResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &crate::msg::QueryMsg::Config {})
        .unwrap();

    (minter_addr, config)
}
