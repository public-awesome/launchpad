use cosmwasm_std::{coins, Addr};
use cw_multi_test::error::Error;
use cw_multi_test::{AppResponse, BankSudo, Contract, ContractWrapper, Executor, SudoMsg};

use sg_multi_test::StargazeApp;
use sg_std::{self, StargazeMsgWrapper};

use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::testing::constants::WHITELIST_AMOUNT;
use anyhow::Error as anyhow_error;
use eyre::Result;
extern crate whitelist_immutable;
use super::test_msgs::InstantiateParams;
use crate::testing::constants::{NATIVE_DENOM, OWNER};
use crate::testing::setup::{
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
        crate::query::query,
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
