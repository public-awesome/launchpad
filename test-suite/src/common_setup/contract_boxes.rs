use cosmwasm_std::testing::{MockApi, MockStorage};
use cosmwasm_std::Empty;
use cw_multi_test::{
    no_init, AppBuilder, BankKeeper, Contract, ContractWrapper, DistributionKeeper, FailingModule,
    GovFailingModule, IbcFailingModule, StakeKeeper, WasmKeeper,
};

use crate::common_setup::keeper::StargazeKeeper;
pub type App<ExecC = Empty, QueryC = Empty> = cw_multi_test::App<
    BankKeeper,
    MockApi,
    MockStorage,
    FailingModule<ExecC, QueryC, Empty>,
    WasmKeeper<ExecC, QueryC>,
    StakeKeeper,
    DistributionKeeper,
    IbcFailingModule,
    GovFailingModule,
    StargazeKeeper,
>;

pub fn custom_mock_app() -> App {
    AppBuilder::new()
        .with_stargate(StargazeKeeper)
        .build(no_init)
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

pub fn contract_open_edition_factory() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        open_edition_factory::contract::execute,
        open_edition_factory::contract::instantiate,
        open_edition_factory::contract::query,
    )
    .with_sudo(open_edition_factory::contract::sudo);
    Box::new(contract)
}

pub fn contract_base_factory() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        base_factory::contract::execute,
        base_factory::contract::instantiate,
        base_factory::contract::query,
    )
    .with_sudo(base_factory::contract::sudo);
    Box::new(contract)
}

pub fn contract_base_minter() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        base_minter::contract::execute,
        base_minter::contract::instantiate,
        base_minter::contract::query,
    )
    .with_reply(base_minter::contract::reply);
    Box::new(contract)
}

pub fn contract_nt_collection() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        sg721_nt::entry::execute,
        sg721_nt::entry::instantiate,
        sg721_nt::entry::query,
    );
    Box::new(contract)
}

pub fn contract_collection_whitelist() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        sg_whitelist::contract::execute,
        sg_whitelist::contract::instantiate,
        sg_whitelist::contract::query,
    );
    Box::new(contract)
}

pub fn contract_open_edition_minter() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        open_edition_minter::contract::execute,
        open_edition_minter::contract::instantiate,
        open_edition_minter::contract::query,
    )
    .with_reply(open_edition_minter::contract::reply);
    Box::new(contract)
}

pub fn contract_vending_minter() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        vending_minter::contract::execute,
        vending_minter::contract::instantiate,
        vending_minter::contract::query,
    )
    .with_reply(vending_minter::contract::reply);
    Box::new(contract)
}

pub fn contract_sg721_base() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        sg721_base::entry::execute,
        sg721_base::entry::instantiate,
        sg721_base::entry::query,
    );
    Box::new(contract)
}

pub fn contract_sg721_updatable() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        sg721_updatable::entry::execute,
        sg721_updatable::entry::instantiate,
        sg721_updatable::entry::query,
    )
    .with_migrate(sg721_updatable::entry::migrate);
    Box::new(contract)
}

pub fn contract_splits() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new_with_empty(
        sg_splits::contract::execute,
        sg_splits::contract::instantiate,
        sg_splits::contract::query,
    );
    Box::new(contract)
}

pub fn contract_group() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new_with_empty(
        cw4_group::contract::execute,
        cw4_group::contract::instantiate,
        cw4_group::contract::query,
    );
    Box::new(contract)
}

pub fn contract_eth_airdrop() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        sg_eth_airdrop::contract::execute,
        sg_eth_airdrop::contract::instantiate,
        sg_eth_airdrop::query::query,
    )
    .with_reply(sg_eth_airdrop::reply::reply);
    Box::new(contract)
}

pub fn contract_whitelist_immutable() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        whitelist_immutable::contract::execute,
        whitelist_immutable::contract::instantiate,
        whitelist_immutable::contract::query,
    );
    Box::new(contract)
}

pub fn contract_whitelist_merkletree() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        whitelist_mtree::contract::execute,
        whitelist_mtree::contract::instantiate,
        whitelist_mtree::contract::query,
    );
    Box::new(contract)
}
