#[cfg(test)]
mod tests {
    use cosmwasm_std::{coin, Addr, Timestamp, Uint128};
    use cw_multi_test::{BankSudo, Contract, ContractWrapper, Executor, SudoMsg};
    use sg721::{CollectionInfo, RoyaltyInfoResponse};
    use sg_multi_test::StargazeApp;
    use sg_std::{StargazeMsgWrapper, GENESIS_MINT_START_TIME};
    use vending::{ExecuteMsg, SudoParams, VendingMinterInitMsg, VendingMinterParams};
    use vending_factory::helpers::FactoryContract;
    use vending_factory::msg::InstantiateMsg as FactoryInstantiateMsg;

    use crate::helpers::Sg721VendingContract;

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
    const CREATION_FEE: u128 = 5_000_000_000;

    fn custom_mock_app() -> StargazeApp {
        StargazeApp::default()
    }

    fn proper_instantiate_factory() -> (StargazeApp, FactoryContract) {
        let mut app = custom_mock_app();
        let factory_id = app.store_code(factory_contract());
        let minter_id = app.store_code(minter_contract());

        let minter_params = VendingMinterParams {
            code_id: minter_id,
            max_token_limit: 10_000,
            max_per_address_limit: 5,
            creation_fee: Uint128::from(CREATION_FEE),
            ..VendingMinterParams::default()
        };

        let mock_params = SudoParams {
            minter_codes: vec![1, 2, 3],
            vending_minter: minter_params,
        };

        let msg = FactoryInstantiateMsg {
            params: mock_params,
        };
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

        let collection_info: CollectionInfo<RoyaltyInfoResponse> = CollectionInfo {
            creator: ADMIN.to_string(),
            description: "description".to_string(),
            image: "https://example.com/image.png".to_string(),
            ..CollectionInfo::default()
        };

        let msg = ExecuteMsg::CreateVendingMinter(VendingMinterInitMsg {
            num_tokens: 1,
            per_address_limit: 5,
            unit_price: coin(10_000_000, NATIVE_DENOM),
            name: "Collection Name".to_string(),
            base_token_uri: "ipfs://test".to_string(),
            start_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME),
            sg721_code_id: sg721_id,
            collection_info,
            ..VendingMinterInitMsg::default()
        });
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
        use cosmwasm_std::{
            coin,
            testing::{mock_dependencies, MockQuerier},
            Empty, Timestamp,
        };
        use cw_multi_test::{BankSudo, SudoMsg};
        use sg721::{CollectionInfo, RoyaltyInfoResponse};
        use sg_std::GENESIS_MINT_START_TIME;
        use vending::{ExecuteMsg, VendingMinterInitMsg};

        use super::*;

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
