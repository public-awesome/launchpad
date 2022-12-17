use cw_multi_test::{Contract, ContractWrapper};
use sg_multi_test::StargazeApp;
use sg_std::StargazeMsgWrapper;

pub fn custom_mock_app() -> StargazeApp {
    StargazeApp::default()
}

pub fn contract_vending_factory() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        vending_factory::contract::execute,
        vending_factory::contract::instantiate,
        vending_factory::contract::query,
    );
    Box::new(contract)
}

pub fn contract_base_factory() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        base_factory::contract::execute,
        base_factory::contract::instantiate,
        base_factory::contract::query,
    )
    .with_sudo(vending_factory::contract::sudo);
    Box::new(contract)
}

pub fn contract_base_minter() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        base_minter::contract::execute,
        base_minter::contract::instantiate,
        base_minter::contract::query,
    )
    .with_reply(base_minter::contract::reply);
    Box::new(contract)
}

pub fn contract_nt_collection() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        sg721_nt::entry::execute,
        sg721_nt::entry::instantiate,
        sg721_nt::entry::query,
    );
    Box::new(contract)
}

pub fn contract_collection_whitelist() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        sg_whitelist::contract::execute,
        sg_whitelist::contract::instantiate,
        sg_whitelist::contract::query,
    );
    Box::new(contract)
}

pub fn contract_vending_minter() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        vending_minter::contract::execute,
        vending_minter::contract::instantiate,
        vending_minter::contract::query,
    )
    .with_reply(vending_minter::contract::reply);
    Box::new(contract)
}

pub fn contract_sg721_base() -> Box<dyn Contract<StargazeMsgWrapper>> {
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

pub fn contract_eth_airdrop() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        sg_eth_airdrop::contract::execute,
        sg_eth_airdrop::contract::instantiate,
        sg_eth_airdrop::query::query,
    )
    .with_reply(sg_eth_airdrop::reply::reply);
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
