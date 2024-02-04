use cosmwasm_std::{coins, Addr, Empty};
use cw_multi_test::{no_init, AppBuilder, BankSudo, Contract, ContractWrapper};
use cw_multi_test::{Executor, SudoMsg};
use sg2::msg::CollectionParams;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
use test_suite::common_setup::contract_boxes::{contract_vending_factory, App};
use test_suite::common_setup::keeper::StargazeKeeper;
use test_suite::common_setup::setup_accounts_and_block::{setup_block_time, INITIAL_BALANCE};
use test_suite::common_setup::setup_minter::common::constants::{CREATION_FEE, MINT_PRICE};
use test_suite::common_setup::setup_minter::common::parse_response::parse_factory_response;
use vending_factory::msg::{
    TokenVaultVendingMinterCreateMsg, TokenVaultVendingMinterInitMsgExtension, VaultInfo,
    VendingMinterInitMsgExtension,
};

use crate::msg::ExecuteMsg;

const FACTORY_ADMIN: &str = "factory_admin";
const CREATOR: &str = "creator";
const BUYER: &str = "buyer";

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

fn setup_app() -> App {
    let mut app = AppBuilder::new()
        .with_stargate(StargazeKeeper)
        .build(no_init);

    app.sudo(SudoMsg::Bank({
        BankSudo::Mint {
            to_address: FACTORY_ADMIN.to_string(),
            amount: coins(INITIAL_BALANCE, NATIVE_DENOM),
        }
    }))
    .unwrap();

    app.sudo(SudoMsg::Bank({
        BankSudo::Mint {
            to_address: CREATOR.to_string(),
            amount: coins(CREATION_FEE, NATIVE_DENOM),
        }
    }))
    .unwrap();

    app.sudo(SudoMsg::Bank({
        BankSudo::Mint {
            to_address: BUYER.to_string(),
            amount: coins(INITIAL_BALANCE, NATIVE_DENOM),
        }
    }))
    .unwrap();

    app
}

fn setup_contracts(app: &mut App) -> (Addr, u64, u64, u64) {
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
            Addr::unchecked(FACTORY_ADMIN),
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

fn create_minter(app: &mut App) -> (Addr, Addr) {
    let (factory_addr, vesting_code_id, _, collection_code_id) = setup_contracts(app);

    let vault_info = VaultInfo {
        vesting_code_id,
        ..VaultInfo::default()
    };

    let init_msg = TokenVaultVendingMinterInitMsgExtension {
        base: VendingMinterInitMsgExtension::default(),
        vault_info,
    };

    let collection_params = sg2::msg::CollectionParams {
        code_id: collection_code_id,
        ..CollectionParams::default()
    };

    let create_minter_msg = TokenVaultVendingMinterCreateMsg {
        init_msg,
        collection_params,
    };

    let msg = vending_factory::msg::ExecuteMsg::CreateTokenVaultMinter(create_minter_msg);

    let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);

    let res = app.execute_contract(
        Addr::unchecked(CREATOR),
        factory_addr.clone(),
        &msg,
        &creation_fee,
    );

    let (minter, collection) = parse_factory_response(&res.unwrap());

    (minter, collection)
}

#[test]
fn proper_initialization() {
    let mut app = setup_app();
    let (minter, collection) = create_minter(&mut app);

    assert_eq!(minter, "contract1".to_string());
    assert_eq!(collection, "contract2".to_string());
}

#[test]
fn mint() {
    let mut app = setup_app();
    let (minter, _) = create_minter(&mut app);

    setup_block_time(&mut app, GENESIS_MINT_START_TIME + 10_000_000, None);

    let mint_msg = ExecuteMsg::Mint {};
    let err = app.execute_contract(
        Addr::unchecked(BUYER),
        minter.clone(),
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    // assert!(res.is_ok());

    assert_eq!(
        err.unwrap_err().source().unwrap().to_string(),
        "Unauthorized".to_string()
    );
}
