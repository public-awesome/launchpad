use crate::testing::setup::msg::MinterCollectionResponse;
use crate::testing::setup::msg::MinterSetupParams;
use crate::testing::setup::setup_contracts::{contract_factory, contract_minter, contract_sg721};
use anyhow::Error;
use cosmwasm_std::{coin, coins, Addr, Timestamp};
use cw_multi_test::{AppResponse, Executor};
use sg2::msg::CreateMinterMsg;
use sg2::msg::{CollectionParams, Sg2ExecuteMsg};
use sg_multi_test::StargazeApp;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
use vending_factory::{
    msg::{VendingMinterCreateMsg, VendingMinterInitMsgExtension},
    state::{ParamsExtension, VendingMinterParams},
};

use super::msg::MinterInstantiateParams;

// pub const LISTING_FEE: u128 = 0;
pub const CREATION_FEE: u128 = 5_000_000_000;
pub const MINT_PRICE: u128 = 100_000_000;

pub const MAX_TOKEN_LIMIT: u32 = 10000;

pub const MIN_MINT_PRICE: u128 = 50_000_000;
pub const AIRDROP_MINT_PRICE: u128 = 0;
pub const MINT_FEE_FAIR_BURN: u64 = 1_000; // 10%
pub const AIRDROP_MINT_FEE_FAIR_BURN: u64 = 10_000; // 100%
pub const SHUFFLE_FEE: u128 = 500_000_000;
pub const MAX_PER_ADDRESS_LIMIT: u32 = 50;

pub fn mock_init_extension(
    splits_addr: Option<String>,
    start_time: Option<Timestamp>,
) -> VendingMinterInitMsgExtension {
    vending_factory::msg::VendingMinterInitMsgExtension {
        base_token_uri: "ipfs://aldkfjads".to_string(),
        payment_address: splits_addr,
        start_time: start_time.unwrap_or(Timestamp::from_nanos(GENESIS_MINT_START_TIME)),
        num_tokens: 100,
        mint_price: coin(MIN_MINT_PRICE, NATIVE_DENOM),
        per_address_limit: 5,
        whitelist: None,
    }
}

pub fn mock_create_minter(
    splits_addr: Option<String>,
    collection_params: CollectionParams,
    start_time: Option<Timestamp>,
) -> VendingMinterCreateMsg {
    VendingMinterCreateMsg {
        init_msg: mock_init_extension(splits_addr, start_time),
        collection_params,
    }
}

pub fn mock_params() -> VendingMinterParams {
    VendingMinterParams {
        code_id: 1,
        creation_fee: coin(CREATION_FEE, NATIVE_DENOM),
        min_mint_price: coin(MIN_MINT_PRICE, NATIVE_DENOM),
        mint_fee_bps: MINT_FEE_FAIR_BURN,
        max_trading_offset_secs: 60 * 60 * 24 * 7,
        extension: ParamsExtension {
            max_token_limit: MAX_TOKEN_LIMIT,
            max_per_address_limit: MAX_PER_ADDRESS_LIMIT,
            airdrop_mint_price: coin(AIRDROP_MINT_PRICE, NATIVE_DENOM),
            airdrop_mint_fee_bps: AIRDROP_MINT_FEE_FAIR_BURN,
            shuffle_fee: coin(SHUFFLE_FEE, NATIVE_DENOM),
        },
    }
}

fn parse_factory_response(res: &AppResponse) -> (Addr, Addr) {
    let events = res.events.clone();
    let mut contract_addrs: Vec<String> = vec![];
    let vector_of_attribute_vectors = events
        .iter()
        .filter(|e| e.ty == "instantiate")
        .map(|v| v.attributes.clone())
        .collect::<Vec<_>>();
    for vector in vector_of_attribute_vectors {
        let contract_addr = vector
            .iter()
            .filter(|a| a.key == "_contract_addr")
            .map(|e| e.value.clone())
            .collect::<Vec<_>>();
        contract_addrs = [contract_addrs.clone(), contract_addr].concat();
    }
    let minter_addr = Addr::unchecked(contract_addrs[0].clone());
    let collection_addr = Addr::unchecked(contract_addrs[1].clone());
    (minter_addr, collection_addr)
}

fn build_init_msg(
    init_msg: Option<VendingMinterInitMsgExtension>,
    mut msg: CreateMinterMsg<VendingMinterInitMsgExtension>,
    num_tokens: u32,
) -> VendingMinterInitMsgExtension {
    match init_msg {
        Some(init_msg_from_params) => init_msg_from_params,
        None => {
            msg.init_msg.mint_price = coin(MINT_PRICE, NATIVE_DENOM);
            msg.init_msg.num_tokens = num_tokens;
            msg.init_msg
        }
    }
}

fn build_collection_response(
    res: Result<AppResponse, Error>,
    factory_addr: Addr,
) -> MinterCollectionResponse {
    match res.is_ok() {
        true => {
            let (minter_addr, collection_addr) = parse_factory_response(&res.unwrap());
            MinterCollectionResponse {
                minter: Some(minter_addr),
                collection: Some(collection_addr),
                factory: Some(factory_addr),
                error: None,
            }
        }
        false => MinterCollectionResponse {
            minter: None,
            collection: None,
            factory: Some(factory_addr),
            error: Some(res.unwrap_err()),
        },
    }
}
// Upload contract code and instantiate minter contract
fn setup_minter_contract(setup_params: MinterSetupParams) -> MinterCollectionResponse {
    let minter_code_id = setup_params.minter_code_id;
    let router = setup_params.router;
    let factory_code_id = setup_params.factory_code_id;
    let sg721_code_id = setup_params.sg721_code_id;
    let minter_admin = setup_params.minter_admin;
    let num_tokens = setup_params.num_tokens;
    let splits_addr = setup_params.splits_addr;
    let collection_params = setup_params.collection_params;
    let start_time = setup_params.start_time;
    let init_msg = setup_params.init_msg;

    let mut params = mock_params();
    params.code_id = minter_code_id;

    let factory_addr = router
        .instantiate_contract(
            factory_code_id,
            minter_admin.clone(),
            &vending_factory::msg::InstantiateMsg { params },
            &[],
            "factory",
            None,
        )
        .unwrap();
    let mut msg = mock_create_minter(splits_addr, collection_params, start_time);
    msg.init_msg = build_init_msg(init_msg, msg.clone(), num_tokens);
    msg.collection_params.code_id = sg721_code_id;
    msg.collection_params.info.creator = minter_admin.to_string();
    let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);
    let msg = Sg2ExecuteMsg::CreateMinter(msg);

    let res = router.execute_contract(minter_admin, factory_addr.clone(), &msg, &creation_fee);
    build_collection_response(res, factory_addr)
}

pub fn get_code_ids(router: &mut StargazeApp) -> (u64, u64, u64) {
    let minter_code_id = router.store_code(contract_minter());
    println!("minter_code_id: {}", minter_code_id);

    let factory_code_id = router.store_code(contract_factory());
    println!("factory_code_id: {}", factory_code_id);

    let sg721_code_id = router.store_code(contract_sg721());
    println!("sg721_code_id: {}", sg721_code_id);
    (minter_code_id, factory_code_id, sg721_code_id)
}

pub fn configure_minter(
    app: &mut StargazeApp,
    minter_admin: Addr,
    collection_params_vec: Vec<CollectionParams>,
    minter_instantiate_params_vec: Vec<MinterInstantiateParams>,
) -> Vec<MinterCollectionResponse> {
    let mut minter_collection_info: Vec<MinterCollectionResponse> = vec![];
    let (minter_code_id, factory_code_id, sg721_code_id) = get_code_ids(app);
    for (index, collection_param) in collection_params_vec.iter().enumerate() {
        let setup_params: MinterSetupParams = MinterSetupParams {
            router: app,
            minter_admin: minter_admin.clone(),
            num_tokens: minter_instantiate_params_vec[index].num_tokens,
            collection_params: collection_param.to_owned(),
            splits_addr: minter_instantiate_params_vec[index].splits_addr.clone(),
            minter_code_id,
            factory_code_id,
            sg721_code_id,
            start_time: minter_instantiate_params_vec[index].start_time,
            init_msg: minter_instantiate_params_vec[index].init_msg.clone(),
        };
        let minter_collection_res = setup_minter_contract(setup_params);
        minter_collection_info.push(minter_collection_res);
    }
    minter_collection_info
}

pub fn minter_params_all(
    num_tokens: u32,
    splits_addr: Option<String>,
    start_time: Option<Timestamp>,
    init_msg: Option<VendingMinterInitMsgExtension>,
) -> MinterInstantiateParams {
    MinterInstantiateParams {
        num_tokens,
        splits_addr,
        start_time,
        init_msg,
    }
}

pub fn minter_params_token(num_tokens: u32) -> MinterInstantiateParams {
    MinterInstantiateParams {
        num_tokens,
        splits_addr: None,
        start_time: None,
        init_msg: None,
    }
}
