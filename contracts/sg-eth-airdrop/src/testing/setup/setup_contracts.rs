use cosmwasm_std::{coins, Addr};
use cw_multi_test::error::Error;
use cw_multi_test::{AppResponse, BankSudo, Contract, ContractWrapper, Executor, SudoMsg};

use sg_multi_test::StargazeApp;
use sg_std::{self, StargazeMsgWrapper};

use crate::msg::{ExecuteMsg, InstantiateMsg};
use eyre::Result;

extern crate whitelist_generic;
use crate::tests_folder::claim_constants::{CONTRACT_CONFIG_PLAINTEXT, NATIVE_DENOM, OWNER};

pub fn custom_mock_app() -> StargazeApp {
    StargazeApp::default()
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
    .with_reply(crate::contract::reply);
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

pub fn instantiate_contract(
    addresses: Vec<String>,
    funds_amount: u128,
    expected_airdrop_contract_id: u64,
    minter_address: Addr,
    admin_account: Addr,
    app: &mut StargazeApp,
) {
    app.sudo(SudoMsg::Bank({
        BankSudo::Mint {
            to_address: admin_account.to_string(),
            amount: coins(funds_amount, NATIVE_DENOM),
        }
    }))
    .map_err(|err| println!("{:?}", err))
    .ok();

    let sg_eth_id = app.store_code(contract());
    let whitelist_code_id = app.store_code(whitelist_generic_contract());
    assert_eq!(sg_eth_id, expected_airdrop_contract_id);
    let msg: InstantiateMsg = InstantiateMsg {
        admin: Addr::unchecked(OWNER),
        claim_msg_plaintext: Addr::unchecked(CONTRACT_CONFIG_PLAINTEXT).into_string(),
        airdrop_amount: 3000,
        addresses,
        whitelist_code_id,
        minter_address,
    };
    let _ = app
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

pub fn instantiate_contract_get_app(
    addresses: Vec<String>,
    funds_amount: u128,
    expected_airdrop_contract_id: u64,
    minter_address: Addr,
) -> StargazeApp {
    let mut app = custom_mock_app();

    instantiate_contract(
        addresses,
        funds_amount,
        expected_airdrop_contract_id,
        minter_address,
        Addr::unchecked(OWNER),
        &mut app,
    );
    app
}

pub fn execute_contract_with_msg(
    msg: ExecuteMsg,
    app: &mut StargazeApp,
    user: Addr,
    target_address: Addr,
) -> Result<AppResponse, Error> {
    let result = app
        .execute_contract(user, target_address, &msg, &[])
        .unwrap();
    Ok(result)
}
