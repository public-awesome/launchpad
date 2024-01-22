#[cfg(test)]
mod tests {
    use crate::common_setup::contract_boxes::{contract_base_factory, custom_mock_app, App};
    use crate::common_setup::setup_minter::base_minter::mock_params::mock_params;
    use base_factory::helpers::FactoryContract;
    use base_factory::msg::InstantiateMsg;
    use cosmwasm_std::Addr;
    use cw_multi_test::Executor;

    const GOVERNANCE: &str = "governance";

    fn proper_instantiate() -> (App, FactoryContract) {
        let mut app = custom_mock_app();
        let factory_id = app.store_code(contract_base_factory());
        let minter_id = 2;

        let mut params = mock_params();
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

    mod init {
        use super::*;

        #[test]
        fn can_init() {
            let (_, factory_contract) = proper_instantiate();
            assert_eq!(factory_contract.addr().to_string(), "contract0");
        }
    }
}
