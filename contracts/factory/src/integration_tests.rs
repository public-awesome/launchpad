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
        )
        .with_reply(crate::contract::reply);
        Box::new(contract)
    }

    pub fn minter_contract() -> Box<dyn Contract<StargazeMsgWrapper>> {
        let contract = ContractWrapper::new(
            minter::contract::execute,
            minter::contract::instantiate,
            minter::contract::query,
        )
        .with_reply(minter::contract::reply);
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

    const GOVERNANCE: &str = "governance";
    const ADMIN: &str = "ADMIN";
    const NATIVE_DENOM: &str = "ustars";
    const CREATION_FEE: u128 = 5_000_000_000;

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
        use cw_multi_test::{BankSudo, SudoMsg};
        use launchpad::VendingMinterInitMsg;
        use sg721::{CollectionInfo, RoyaltyInfoResponse};
        use sg_std::GENESIS_MINT_START_TIME;

        use super::*;
        use crate::msg::ExecuteMsg;

        #[test]
        fn create_vending_minter() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let sg721_id = app.store_code(sg721_vending_contract());

            let collection_info: CollectionInfo<RoyaltyInfoResponse> = CollectionInfo {
                creator: "creator".to_string(),
                description: "description".to_string(),
                image: "https://example.com/image.png".to_string(),
                ..CollectionInfo::default()
            };

            let msg = ExecuteMsg::CreateVendingMinter(VendingMinterInitMsg {
                num_tokens: 1,
                per_address_limit: 5,
                unit_price: coin(10_000_000, NATIVE_DENOM),
                name: "Test Name".to_string(),
                base_token_uri: "ipfs://test".to_string(),
                start_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME),
                sg721_code_id: sg721_id,
                collection_info,
                ..VendingMinterInitMsg::default()
            });
            let creation_fee = coin(CREATION_FEE, NATIVE_DENOM);

            app.sudo(SudoMsg::Bank(BankSudo::Mint {
                to_address: GOVERNANCE.to_string(),
                amount: vec![creation_fee.clone()],
            }))
            .unwrap();

            let bal = app.wrap().query_all_balances(GOVERNANCE).unwrap();
            assert_eq!(bal, vec![creation_fee.clone()]);

            let cosmos_msg = cw_template_contract
                .call_with_funds(msg, creation_fee)
                .unwrap();

            app.execute(Addr::unchecked(GOVERNANCE), cosmos_msg)
                .unwrap();
        }
    }
}
