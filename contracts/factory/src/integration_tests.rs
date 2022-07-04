#[cfg(test)]
mod tests {
    use crate::helpers::CwTemplateContract;
    use crate::msg::InstantiateMsg;
    use cosmwasm_std::Addr;
    use cw_multi_test::{Contract, ContractWrapper, Executor};
    use launchpad::{SudoParams, VendingMinterParams};
    use sg_multi_test::StargazeApp;
    use sg_std::StargazeMsgWrapper;

    pub fn contract_template() -> Box<dyn Contract<StargazeMsgWrapper>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    pub fn minter_contract() -> Box<dyn Contract<StargazeMsgWrapper>> {
        let contract = ContractWrapper::new(
            minter::contract::execute,
            minter::contract::instantiate,
            minter::contract::query,
        );
        Box::new(contract)
    }

    pub fn sg721_vending_contract() -> Box<dyn Contract<StargazeMsgWrapper>> {
        let contract = ContractWrapper::new(
            sg721_vending::contract::execute,
            sg721_vending::contract::instantiate,
            sg721_vending::contract::query,
        );
        Box::new(contract)
    }

    const USER: &str = "USER";
    const ADMIN: &str = "ADMIN";
    const NATIVE_DENOM: &str = "ustars";

    fn custom_mock_app() -> StargazeApp {
        StargazeApp::default()
    }

    fn proper_instantiate() -> (StargazeApp, CwTemplateContract) {
        let mut app = custom_mock_app();
        let cw_template_id = app.store_code(contract_template());
        let minter_id = app.store_code(minter_contract());

        let minter_params = VendingMinterParams {
            code_id: minter_id,
            max_token_limit: 10_000,
            max_per_address_limit: 5,
            // min_mint_price: todo!(),
            // airdrop_mint_price: todo!(),
            // mint_fee_percent: todo!(),
            // airdrop_mint_fee_percent: todo!(),
            // shuffle_fee: todo!(),
            ..VendingMinterParams::default()
        };

        let mock_params = SudoParams {
            minter_codes: vec![1, 2, 3],
            vending_minter: minter_params,
        };

        let msg = InstantiateMsg {
            params: mock_params,
        };
        let cw_template_contract_addr = app
            .instantiate_contract(
                cw_template_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "factory",
                None,
            )
            .unwrap();

        let cw_template_contract = CwTemplateContract(cw_template_contract_addr);

        (app, cw_template_contract)
    }

    mod params {
        use cosmwasm_std::{coin, Timestamp};
        use launchpad::VendingMinterInitMsg;
        use sg_std::GENESIS_MINT_START_TIME;

        use super::*;
        use crate::msg::ExecuteMsg;

        #[test]
        fn create_vending_minter() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let sg721_id = app.store_code(sg721_vending_contract());

            let msg = ExecuteMsg::CreateVendingMinter(VendingMinterInitMsg {
                num_tokens: 1,
                per_address_limit: 5,
                unit_price: coin(10_000_000, NATIVE_DENOM),
                name: "Test Name".to_string(),
                base_token_uri: "ipfs://test".to_string(),
                start_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME),
                sg721_code_id: sg721_id,
                ..VendingMinterInitMsg::default()
            });
            // println!("{:?}", msg);
            let cosmos_msg = cw_template_contract.call(msg).unwrap();
            app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();
        }
    }
}
