use cosmwasm_std::{coin, coins, Empty, Timestamp};
use cw_multi_test::Executor;
use cw_multi_test::{App, BasicApp, Contract, ContractWrapper};
use sg2::tests::mock_collection_params_1;
// use sg_multi_test::StargazeApp;
// use sg_std::StargazeMsgWrapper;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
use test_suite::common_setup::setup_accounts_and_block::setup_accounts;
use test_suite::common_setup::setup_minter::vending_minter::mock_params::mock_params;
use test_suite::common_setup::setup_minter::{
    common::constants::CREATION_FEE, vending_minter::mock_params::mock_init_extension,
};
use vending_factory::msg::{
    TokenVaultVendingMinterCreateMsg, TokenVaultVendingMinterInitMsgExtension, VaultInfo,
};

fn cw_vesting_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw_vesting::contract::execute,
        cw_vesting::contract::instantiate,
        cw_vesting::contract::query,
    );
    Box::new(contract)
}

pub fn contract_vending_factory() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        vending_factory::contract::execute,
        vending_factory::contract::instantiate,
        vending_factory::contract::query,
    )
    .with_sudo(vending_factory::contract::sudo);
    Box::new(contract)
}

// fn cw_vesting_contract() -> Box<dyn Contract<StargazeMsgWrapper>> {
//     let contract = ContractWrapper::new(
//         cw_vesting::contract::execute,
//         cw_vesting::contract::instantiate,
//         cw_vesting::contract::query,
//     );
//     Box::new(contract)
// }

// pub fn custom_mock_app() -> StargazeApp {
//     StargazeApp::default()
// }

#[test]
fn proper_initialization() {
    let start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    let collection_params = mock_collection_params_1(Some(start_time));
    let params = mock_params(None);

    // let mut app = custom_mock_app();
    let mut app = BasicApp::default();

    let factory_code_id = app.store_code(contract_vending_factory());
    let vesting_code_id = app.store_code(cw_vesting_contract());
    let (creator, buyer) = setup_accounts(&mut app);

    let factory_addr = app
        .instantiate_contract(
            factory_code_id,
            creator.clone(),
            &vending_factory::msg::InstantiateMsg { params },
            &[],
            "factory",
            None,
        )
        .unwrap();

    let base = mock_init_extension(None, None);

    let vault_info = VaultInfo {
        token_balance: coin(100u128, NATIVE_DENOM),
        vesting_schedule: cw_vesting::vesting::Schedule::SaturatingLinear,
        vesting_duration_seconds: 1000,
        unbonding_duration_seconds: 0,
        vesting_code_id: 8,
    };

    let init_msg = TokenVaultVendingMinterInitMsgExtension { base, vault_info };

    let create_minter_msg = TokenVaultVendingMinterCreateMsg {
        init_msg,
        collection_params,
    };

    let msg = vending_factory::msg::ExecuteMsg::CreateTokenVaultMinter(create_minter_msg);

    let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);

    let res = app.execute_contract(creator, factory_addr.clone(), &msg, &creation_fee);

    assert!(res.is_ok())
}
