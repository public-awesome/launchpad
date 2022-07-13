#[cfg(test)]
mod tests {
    use crate::helpers::Sg721VendingContract;
    use cosmwasm_std::{coin, Addr};
    use cw_multi_test::{BankSudo, Contract, ContractWrapper, Executor, SudoMsg};
    use sg_multi_test::StargazeApp;
    use sg_std::StargazeMsgWrapper;
    use vending::tests::{mock_create_minter, mock_params, CREATION_FEE};
    use vending::ExecuteMsg;
    use vending_factory::helpers::FactoryContract;
    use vending_factory::msg::InstantiateMsg as FactoryInstantiateMsg;

    pub fn factory_contract() -> Box<dyn Contract<StargazeMsgWrapper>> {
        let contract = ContractWrapper::new(
            vending_factory::contract::execute,
            vending_factory::contract::instantiate,
            vending_factory::contract::query,
        )
        .with_reply(vending_factory::contract::reply);
        Box::new(contract)
    }

    pub fn minter_contract() -> Box<dyn Contract<StargazeMsgWrapper>> {
        let contract = ContractWrapper::new(
            vending_minter::contract::execute,
            vending_minter::contract::instantiate,
            vending_minter::contract::query,
        )
        .with_reply(vending_minter::contract::reply);
        Box::new(contract)
    }

    pub fn sg721_vending_contract() -> Box<dyn Contract<StargazeMsgWrapper>> {
        let contract = ContractWrapper::new(
            crate::entry::execute,
            crate::entry::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    const GOVERNANCE: &str = "governance";
    const ADMIN: &str = "admin";
    const NATIVE_DENOM: &str = "ustars";

    fn custom_mock_app() -> StargazeApp {
        StargazeApp::default()
    }

    fn proper_instantiate_factory() -> (StargazeApp, FactoryContract) {
        let mut app = custom_mock_app();
        let factory_id = app.store_code(factory_contract());
        let minter_id = app.store_code(minter_contract());

        let mut params = mock_params();
        params.code_id = minter_id;

        let msg = FactoryInstantiateMsg { params };
        let factory_addr = app
            .instantiate_contract(
                factory_id,
                Addr::unchecked(GOVERNANCE),
                &msg,
                &[],
                "factory",
                Some(GOVERNANCE.to_string()),
            )
            .unwrap();

        let factory_contract = FactoryContract(factory_addr);

        (app, factory_contract)
    }

    fn proper_instantiate() -> (StargazeApp, Sg721VendingContract) {
        let (mut app, factory_contract) = proper_instantiate_factory();
        let sg721_id = app.store_code(sg721_vending_contract());

        let mut m = mock_create_minter();
        m.collection_params.code_id = sg721_id;
        let msg = ExecuteMsg::CreateVendingMinter(m);

        let creation_fee = coin(CREATION_FEE, NATIVE_DENOM);

        app.sudo(SudoMsg::Bank(BankSudo::Mint {
            to_address: ADMIN.to_string(),
            amount: vec![creation_fee.clone()],
        }))
        .unwrap();

        let bal = app.wrap().query_all_balances(ADMIN).unwrap();
        assert_eq!(bal, vec![creation_fee.clone()]);

        let cosmos_msg = factory_contract.call_with_funds(msg, creation_fee).unwrap();

        let res = app.execute(Addr::unchecked(ADMIN), cosmos_msg);
        assert!(res.is_ok());

        // can also get this address from the events from the res above
        let contract = Sg721VendingContract(Addr::unchecked("contract1"));

        (app, contract)
    }

    mod init {
        use super::*;
        use cosmwasm_std::{
            testing::{mock_dependencies, MockQuerier},
            Empty,
        };

        #[test]
        fn create_sg721_vending_collection() {
            let deps = mock_dependencies();

            let (_, contract) = proper_instantiate();

            // query contract...
            let res = contract.num_tokens::<MockQuerier, Empty>(&deps.querier);
            // println!("{:?}", res);
        }
    }
}
