#[cfg(test)]
mod tests {
    use crate::helpers::Sg721VendingContract;
    use cosmwasm_std::{coin, Addr, Decimal, Uint128};
    use cw_multi_test::{BankSudo, Contract, ContractWrapper, Executor, SudoMsg};
    use sg_multi_test::StargazeApp;
    use sg_std::StargazeMsgWrapper;
    use vending::tests::{
        mock_init_msg, AIRDROP_MINT_FEE_BPS, AIRDROP_MINT_PRICE, CREATION_FEE, MINT_FEE_BPS,
        MIN_MINT_PRICE, SHUFFLE_FEE,
    };
    use vending::{ExecuteMsg, VendingMinterParams};
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

        let minter_params = VendingMinterParams {
            code_id: minter_id,
            max_token_limit: 10_000,
            max_per_address_limit: 5,
            creation_fee: Uint128::from(CREATION_FEE),
            min_mint_price: Uint128::from(MIN_MINT_PRICE),
            airdrop_mint_price: Uint128::from(AIRDROP_MINT_PRICE),
            mint_fee_percent: Decimal::percent(MINT_FEE_BPS),
            airdrop_mint_fee_percent: Decimal::percent(AIRDROP_MINT_FEE_BPS),
            shuffle_fee: Uint128::from(SHUFFLE_FEE),
        };

        let msg = FactoryInstantiateMsg {
            params: minter_params,
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

        let mut m = mock_init_msg();
        m.sg721_code_id = sg721_id;
        m.factory = factory_contract.addr().to_string();
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
        println!("{:?}", res);
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
