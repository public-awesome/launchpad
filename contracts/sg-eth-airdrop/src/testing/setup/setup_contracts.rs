use cosmwasm_std::{coins, Addr};
use cw_multi_test::error::Error;
use cw_multi_test::{AppResponse, BankSudo, Contract, ContractWrapper, Executor, SudoMsg};

use sg_multi_test::StargazeApp;
use sg_std::{self, StargazeMsgWrapper};

use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::tests_folder::constants::WHITELIST_AMOUNT;
use eyre::Result;

extern crate whitelist_immutable;
use super::test_msgs::InstantiateParams;
use crate::tests_folder::constants::{CONTRACT_CONFIG_PLAINTEXT, NATIVE_DENOM, OWNER};
use crate::tests_folder::tests_setup::{
    mock_minter_execute, mock_minter_instantiate, mock_minter_query, mock_whitelist_execute,
    mock_whitelist_instantiate, mock_whitelist_query,
};

pub fn custom_mock_app() -> StargazeApp {
    StargazeApp::default()
}

pub fn mock_minter() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        mock_minter_execute,
        mock_minter_instantiate,
        mock_minter_query,
    );
    Box::new(contract)
}

pub fn mock_whitelist() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        mock_whitelist_execute,
        mock_whitelist_instantiate,
        mock_whitelist_query,
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

pub fn contract() -> Box<dyn Contract<sg_std::StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    )
    .with_reply(crate::reply::reply);
    Box::new(contract)
}

pub fn whitelist_immutable_contract() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        whitelist_immutable::contract::execute,
        whitelist_immutable::contract::instantiate,
        whitelist_immutable::contract::query,
    );
    Box::new(contract)
}

pub fn instantiate_contract(params: InstantiateParams) {
    let addresses = params.addresses;
    let minter_address = params.minter_address;
    let admin_account = params.admin_account;
    let funds_amount = params.funds_amount;
    let per_address_limit = params.per_address_limit;
    println!("funds amount is {:?}", funds_amount);
    params
        .app
        .sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: admin_account.to_string(),
                amount: coins(params.funds_amount, NATIVE_DENOM),
            }
        }))
        .map_err(|err| println!("{:?}", err))
        .ok();

    let sg_eth_id = params.app.store_code(contract());
    let whitelist_code_id = params.app.store_code(whitelist_immutable_contract());
    assert_eq!(sg_eth_id, params.expected_airdrop_contract_id);

    let msg: InstantiateMsg = InstantiateMsg {
        admin: Addr::unchecked(OWNER),
        claim_msg_plaintext: Addr::unchecked(CONTRACT_CONFIG_PLAINTEXT).into_string(),
        airdrop_amount: WHITELIST_AMOUNT,
        addresses,
        whitelist_code_id,
        minter_address,
        per_address_limit,
    };
    let _ = params
        .app
        .instantiate_contract(
            sg_eth_id,
            Addr::unchecked(admin_account.clone()),
            &msg,
            &coins(funds_amount, NATIVE_DENOM),
            "sg-eg-airdrop",
            Some(Addr::unchecked(admin_account).to_string()),
        )
        .unwrap();
}

pub fn execute_contract_with_msg(
    msg: ExecuteMsg,
    app: &mut StargazeApp,
    user: Addr,
    target_address: Addr,
) -> Result<AppResponse, Error> {
    let result = app.execute_contract(user, target_address, &msg, &[]);
    Ok(result.unwrap())
}

pub fn execute_contract_error_with_msg(
    msg: ExecuteMsg,
    app: &mut StargazeApp,
    user: Addr,
    target_address: Addr,
) -> String {
    let result = app
        .execute_contract(user, target_address, &msg, &[])
        .unwrap_err();
    result.root_cause().to_string()
}
