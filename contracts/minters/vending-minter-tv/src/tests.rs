use crate::msg::ExecuteMsg;
use cosmwasm_std::testing::MockStorage;
use cosmwasm_std::{coins, Addr, Empty, Timestamp};
use cw_multi_test::addons::{MockAddressGenerator, MockApiBech32};
use cw_multi_test::{
    no_init, AppBuilder, BankKeeper, BankSudo, Contract, ContractWrapper, DistributionKeeper,
    FailingModule, GovFailingModule, IbcFailingModule, StakeKeeper, WasmKeeper,
};
use cw_multi_test::{Executor, SudoMsg};
use sg2::msg::CollectionParams;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
use test_suite::common_setup::contract_boxes::contract_vending_factory;
use test_suite::common_setup::keeper::StargazeKeeper;
use test_suite::common_setup::setup_accounts_and_block::INITIAL_BALANCE;
use test_suite::common_setup::setup_minter::common::constants::{CREATION_FEE, MINT_PRICE};
use test_suite::common_setup::setup_minter::common::parse_response::parse_factory_response;
use vending_factory::msg::{
    TokenVaultVendingMinterCreateMsg, TokenVaultVendingMinterInitMsgExtension, VaultInfo,
    VendingMinterInitMsgExtension,
};

// const BUYER: &str = "buyer";

fn cw_vesting_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw_vesting::contract::execute,
        cw_vesting::contract::instantiate,
        cw_vesting::contract::query,
    );
    Box::new(contract)
}

fn contract_vending_minter_tv() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    )
    .with_reply(crate::contract::reply);
    Box::new(contract)
}

fn contract_tv_collection() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        sg721_tv::entry::execute,
        sg721_tv::entry::instantiate,
        sg721_tv::entry::query,
    );
    Box::new(contract)
}

pub type App<ExecC = Empty, QueryC = Empty> = cw_multi_test::App<
    BankKeeper,
    MockApiBech32,
    MockStorage,
    FailingModule<ExecC, QueryC, Empty>,
    WasmKeeper<ExecC, QueryC>,
    StakeKeeper,
    DistributionKeeper,
    IbcFailingModule,
    GovFailingModule,
    StargazeKeeper,
>;

fn setup_app() -> (App, Addr, Addr, Addr) {
    let mut app = AppBuilder::default()
        .with_api(MockApiBech32::new("stars"))
        .with_wasm(WasmKeeper::default().with_address_generator(MockAddressGenerator))
        .with_stargate(StargazeKeeper)
        .build(no_init);

    let factory_admin = app.api().addr_make("factory_admin");
    let creator = app.api().addr_make("creator");
    let buyer = app.api().addr_make("buyer");

    app.sudo(SudoMsg::Bank({
        BankSudo::Mint {
            to_address: factory_admin.to_string(),
            amount: coins(INITIAL_BALANCE, NATIVE_DENOM),
        }
    }))
    .unwrap();

    app.sudo(SudoMsg::Bank({
        BankSudo::Mint {
            to_address: creator.to_string(),
            amount: coins(CREATION_FEE, NATIVE_DENOM),
        }
    }))
    .unwrap();

    app.sudo(SudoMsg::Bank({
        BankSudo::Mint {
            to_address: buyer.to_string(),
            amount: coins(INITIAL_BALANCE, NATIVE_DENOM),
        }
    }))
    .unwrap();

    (app, factory_admin, creator, buyer)
}

fn setup_contracts(app: &mut App, factory_admin: Addr) -> (Addr, u64, u64, u64) {
    let factory_code_id = app.store_code(contract_vending_factory());
    let vesting_code_id = app.store_code(cw_vesting_contract());
    let vending_code_id = app.store_code(contract_vending_minter_tv());
    let collection_code_id = app.store_code(contract_tv_collection());

    let mut init_msg = vending_factory::msg::InstantiateMsg::default();
    init_msg.params.code_id = vending_code_id;
    init_msg.params.allowed_sg721_code_ids = vec![collection_code_id];

    let factory_addr = app
        .instantiate_contract(
            factory_code_id,
            factory_admin,
            &init_msg,
            &[],
            "factory",
            None,
        )
        .unwrap();

    (
        factory_addr,
        vesting_code_id,
        vending_code_id,
        collection_code_id,
    )
}

fn create_minter(app: &mut App, factory_admin: Addr, creator: Addr) -> (Addr, Addr) {
    let (factory_addr, vesting_code_id, _, collection_code_id) =
        setup_contracts(app, factory_admin);

    let vault_info = VaultInfo {
        vesting_code_id,
        ..VaultInfo::default()
    };

    let init_msg = TokenVaultVendingMinterInitMsgExtension {
        base: VendingMinterInitMsgExtension::default(),
        vault_info,
    };

    let mut collection_params = sg2::msg::CollectionParams {
        code_id: collection_code_id,
        ..CollectionParams::default()
    };
    collection_params.info.creator = creator.to_string();

    let create_minter_msg = TokenVaultVendingMinterCreateMsg {
        init_msg,
        collection_params,
    };

    let msg = vending_factory::msg::ExecuteMsg::CreateTokenVaultMinter(create_minter_msg);

    let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);

    let res = app.execute_contract(creator, factory_addr.clone(), &msg, &creation_fee);

    let (minter, collection) = parse_factory_response(&res.unwrap());

    // send token vault funds to the minter
    app.sudo(SudoMsg::Bank({
        BankSudo::Mint {
            to_address: minter.to_string(),
            amount: coins(INITIAL_BALANCE, NATIVE_DENOM),
        }
    }))
    .unwrap();

    (minter, collection)
}

#[test]
fn proper_initialization() {
    let (mut app, factory_admin, creator, _) = setup_app();
    create_minter(&mut app, factory_admin, creator);
}

pub fn setup_block_time(router: &mut App, nanos: u64, height: Option<u64>) {
    let mut block = router.block_info();
    block.time = Timestamp::from_nanos(nanos);
    if let Some(h) = height {
        block.height = h;
    }
    router.set_block(block);
}

#[test]
fn mint() {
    let (mut app, factory_admin, creator, buyer) = setup_app();
    let (minter, _) = create_minter(&mut app, factory_admin, creator);

    setup_block_time(&mut app, GENESIS_MINT_START_TIME + 10_000_000, None);

    let mint_msg = ExecuteMsg::Mint {};
    let res = app.execute_contract(
        buyer,
        minter.clone(),
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_ok());
}
