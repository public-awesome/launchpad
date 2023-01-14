use cosmwasm_std::Empty;
use cw_multi_test::{Contract, ContractWrapper};

pub fn contract_group() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw4_group::contract::execute,
        cw4_group::contract::instantiate,
        cw4_group::contract::query,
    );
    Box::new(contract)
}

pub fn contract_splits() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        sg_splits::contract::execute,
        sg_splits::contract::instantiate,
        sg_splits::contract::query,
    )
    .with_reply(sg_splits::contract::reply);
    Box::new(contract)
}
