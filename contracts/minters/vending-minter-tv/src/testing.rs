use cosmwasm_std::{coin, coins, Addr, Timestamp};
use cw_multi_test::Executor;
use sg2::tests::mock_collection_params_1;
use sg_multi_test::StargazeApp;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
use test_suite::common_setup::setup_minter::{
    common::constants::CREATION_FEE, vending_minter::mock_params::mock_init_extension,
};
use test_suite::common_setup::{
    contract_boxes::contract_vending_factory,
    setup_minter::vending_minter::mock_params::mock_params,
};
use vending_factory::msg::{
    TokenVaultVendingMinterCreateMsg, TokenVaultVendingMinterInitMsgExtension, VaultInfo,
};

pub fn custom_mock_app() -> StargazeApp {
    StargazeApp::default()
}

#[test]
fn proper_initialization() {
    let start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    let collection_params = mock_collection_params_1(Some(start_time));
    let params = mock_params(None);

    let mut app = custom_mock_app();
    let factory_code_id = app.store_code(contract_vending_factory());
    let minter_admin = Addr::unchecked("minter-admin");

    let factory_addr = app
        .instantiate_contract(
            factory_code_id,
            minter_admin.clone(),
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

    // TODO: need to add funds to admin

    let res = app.execute_contract(minter_admin, factory_addr.clone(), &msg, &creation_fee);

    assert!(res.is_ok())
}
