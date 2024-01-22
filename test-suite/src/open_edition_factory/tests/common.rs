use crate::common_setup::contract_boxes::{contract_open_edition_factory, custom_mock_app, App};
use crate::common_setup::setup_minter::open_edition_minter::mock_params::mock_params_proper;
use cosmwasm_std::Addr;
use cw_multi_test::Executor;
use open_edition_factory::helpers::FactoryContract;
use open_edition_factory::msg::InstantiateMsg;

const GOVERNANCE: &str = "governance";

pub fn proper_instantiate() -> (App, FactoryContract) {
    let mut app = custom_mock_app();
    let factory_id = app.store_code(contract_open_edition_factory());
    let minter_id = 2;

    let mut params = mock_params_proper();
    params.code_id = minter_id;

    let factory_contract_addr = app
        .instantiate_contract(
            factory_id,
            Addr::unchecked(GOVERNANCE),
            &InstantiateMsg { params },
            &[],
            "factory",
            None,
        )
        .unwrap();

    (app, FactoryContract(factory_contract_addr))
}
