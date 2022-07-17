#[cfg(test)]
mod tests {
    use crate::helpers::FactoryContract;
    use crate::msg::InstantiateMsg;
    use cosmwasm_std::Addr;
    use cw_multi_test::{Contract, ContractWrapper, Executor};
    use sg_multi_test::StargazeApp;
    use sg_std::StargazeMsgWrapper;
    use vending::tests::{mock_params, CREATION_FEE};

    pub fn factory_contract() -> Box<dyn Contract<StargazeMsgWrapper>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        )
        .with_sudo(crate::contract::sudo)
        .with_reply(crate::contract::reply);
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
            sg721_vending::entry::execute,
            sg721_vending::entry::instantiate,
            sg721_vending::contract::query,
        );
        Box::new(contract)
    }

    const GOVERNANCE: &str = "governance";
    const ADMIN: &str = "admin";
    const NATIVE_DENOM: &str = "ustars";

    fn custom_mock_app() -> StargazeApp {
        StargazeApp::default()
    }

    fn proper_instantiate() -> (StargazeApp, FactoryContract) {
        let mut app = custom_mock_app();
        let factory_id = app.store_code(factory_contract());
        let minter_id = app.store_code(minter_contract());

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

    mod execute {
        use crate::msg::{QueryMsg, SudoMsg};
        use cosmwasm_std::coin;
        use cw_multi_test::{BankSudo, SudoMsg as CwSudoMsg};
        use minters::MinterStatusResponse;
        use vending::{tests::mock_create_minter, ExecuteMsg, ParamsResponse};

        use super::*;

        #[test]
        fn create_vending_minter_and_launch_collection() {
            let (mut app, factory_contract) = proper_instantiate();
            let sg721_id = app.store_code(sg721_vending_contract());
            let minter = "contract1".to_string();

            let mut m = mock_create_minter();
            m.collection_params.code_id = sg721_id;
            let msg = ExecuteMsg::CreateVendingMinter(m);

            let creation_fee = coin(CREATION_FEE, NATIVE_DENOM);

            app.sudo(CwSudoMsg::Bank(BankSudo::Mint {
                to_address: ADMIN.to_string(),
                amount: vec![creation_fee.clone()],
            }))
            .unwrap();

            let bal = app.wrap().query_all_balances(ADMIN).unwrap();
            assert_eq!(bal, vec![creation_fee.clone()]);

            let cosmos_msg = factory_contract.call_with_funds(msg, creation_fee).unwrap();

            app.execute(Addr::unchecked(ADMIN), cosmos_msg).unwrap();

            // query to see if minter default status is set (unverified and unblocked)
            let query_minter_msg = QueryMsg::MinterStatus {
                minter: minter.clone(),
            };
            let res: MinterStatusResponse = app
                .wrap()
                .query_wasm_smart(factory_contract.addr(), &query_minter_msg)
                .unwrap();
            assert!(!res.minter.verified);

            // test sudo
            let msg = SudoMsg::UpdateMinterStatus {
                minter: minter.clone(),
                verified: true,
                blocked: false,
            };
            let res = app.wasm_sudo(factory_contract.addr(), &msg);
            assert!(res.is_ok());

            // query to see if it worked
            let query_minter_msg = QueryMsg::MinterStatus { minter };
            let res: MinterStatusResponse = app
                .wrap()
                .query_wasm_smart(factory_contract.addr(), &query_minter_msg)
                .unwrap();
            assert!(res.minter.verified);

            // query params from factory
            let query_params_msg = QueryMsg::Params {};
            let res: ParamsResponse = app
                .wrap()
                .query_wasm_smart(factory_contract.addr(), &query_params_msg)
                .unwrap();
            assert_eq!(res.params.code_id, 2);
        }
    }
}
