use crate::msg::InstantiateMsg;
use crate::{contract::instantiate, msg::VaultInfo};
use cosmwasm_std::{
    coin, coins,
    testing::{mock_dependencies_with_balance, mock_env, mock_info},
    Addr, Timestamp,
};
use cw_multi_test::{AppResponse, Executor};
use sg2::tests::mock_collection_params_1;
use sg_multi_test::StargazeApp;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
use test_suite::common_setup::{
    contract_boxes::contract_vending_factory,
    setup_accounts_and_block::INITIAL_BALANCE,
    setup_minter::vending_minter::mock_params::{mock_create_minter, mock_params},
};

pub fn custom_mock_app() -> StargazeApp {
    StargazeApp::default()
}

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    let start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    let collection_params = mock_collection_params_1(Some(start_time));

    let create_msg = mock_create_minter(None, collection_params.clone(), None);
    let params = mock_params(None);

    let vault_info = VaultInfo {
        token_balance: coin(100u128, NATIVE_DENOM),
        vesting_schedule: cw_vesting::vesting::Schedule::SaturatingLinear,
        vesting_duration_seconds: 1000,
        unbonding_duration_seconds: 0,
        vesting_code_id: 8,
    };

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

    // let msg = InstantiateMsg {
    //     create_msg,
    //     params,
    //     vault_info,
    // };

    // let info = mock_info("creator", &coins(INITIAL_BALANCE, NATIVE_DENOM));

    // let res = instantiate(deps.as_mut(), mock_env(), info, msg);

    // assert!(res.is_ok())
}
