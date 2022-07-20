#[cfg(test)]
mod tests {
    use crate::msg::{InstantiateMsg, VendingMinterCreateMsg, VendingMinterInitMsgExtension};
    use crate::state::ParamsExtension;
    use crate::{helpers::FactoryContract, state::VendingMinterParams};
    use cosmwasm_std::{coin, Addr, Timestamp};
    use cw_multi_test::{Contract, ContractWrapper, Executor};
    use sg2::tests::mock_collection_params;
    use sg_multi_test::StargazeApp;
    use sg_std::{StargazeMsgWrapper, GENESIS_MINT_START_TIME};

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

    pub const CREATION_FEE: u128 = 5_000_000_000;
    pub const MIN_MINT_PRICE: u128 = 50_000_000;
    pub const AIRDROP_MINT_PRICE: u128 = 15_000_000;
    pub const MINT_FEE_BPS: u64 = 1_000; // 10%
    pub const AIRDROP_MINT_FEE_BPS: u64 = 10_000; // 100%
    pub const SHUFFLE_FEE: u128 = 500_000_000;
    pub const MAX_TOKEN_LIMIT: u32 = 10_000;
    pub const MAX_PER_ADDRESS_LIMIT: u32 = 50;

    fn custom_mock_app() -> StargazeApp {
        StargazeApp::default()
    }

    // TODO: dupe, make DRY
    pub fn mock_params() -> VendingMinterParams {
        VendingMinterParams {
            code_id: 1,
            creation_fee: coin(CREATION_FEE, NATIVE_DENOM),
            min_mint_price: coin(MIN_MINT_PRICE, NATIVE_DENOM),
            mint_fee_bps: MINT_FEE_BPS,
            extension: ParamsExtension {
                max_token_limit: MAX_TOKEN_LIMIT,
                max_per_address_limit: MAX_PER_ADDRESS_LIMIT,
                airdrop_mint_price: coin(AIRDROP_MINT_PRICE, NATIVE_DENOM),
                airdrop_mint_fee_bps: AIRDROP_MINT_FEE_BPS,
                shuffle_fee: coin(SHUFFLE_FEE, NATIVE_DENOM),
            },
        }
    }

    // TODO: dupe, make DRY
    pub fn mock_init_extension() -> VendingMinterInitMsgExtension {
        VendingMinterInitMsgExtension {
            base_token_uri: "ipfs://aldkfjads".to_string(),
            start_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME),
            num_tokens: 100,
            unit_price: coin(MIN_MINT_PRICE, NATIVE_DENOM),
            per_address_limit: 5,
            whitelist: None,
        }
    }

    // TODO: dupe, make DRY
    pub fn mock_create_minter() -> VendingMinterCreateMsg {
        VendingMinterCreateMsg {
            init_msg: mock_init_extension(),
            collection_params: mock_collection_params(),
        }
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
        use crate::msg::{ExecuteMsg, ParamsResponse, SudoMsg};
        use cosmwasm_std::coin;
        use cw_multi_test::{BankSudo, SudoMsg as CwSudoMsg};
        use sg2::query::{MinterStatusResponse, Sg2QueryMsg};

        use super::*;

        #[test]
        fn create_vending_minter_and_launch_collection() {
            let (mut app, factory_contract) = proper_instantiate();
            let sg721_id = app.store_code(sg721_vending_contract());
            let minter = "contract1".to_string();

            let mut m = mock_create_minter();
            m.collection_params.code_id = sg721_id;
            let msg = ExecuteMsg::CreateMinter(m);

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
            let query_minter_msg = Sg2QueryMsg::MinterStatus {
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
            let query_minter_msg = Sg2QueryMsg::MinterStatus { minter };
            let res: MinterStatusResponse = app
                .wrap()
                .query_wasm_smart(factory_contract.addr(), &query_minter_msg)
                .unwrap();
            assert!(res.minter.verified);

            // query params from factory
            let query_params_msg = Sg2QueryMsg::Params {};
            let res: ParamsResponse = app
                .wrap()
                .query_wasm_smart(factory_contract.addr(), &query_params_msg)
                .unwrap();
            assert_eq!(res.params.code_id, 2);
        }
    }
}
