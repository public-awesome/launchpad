#[cfg(test)]
mod tests {
    use cosmwasm_std::{coin, Addr, Timestamp};
    use cw721::NumTokensResponse;
    use cw_multi_test::{BankSudo, Contract, ContractWrapper, Executor, SudoMsg};
    use sg2::tests::mock_collection_params;
    use sg721::ExecuteMsg as Sg721ExecuteMsg;
    use sg721::{CollectionInfo, InstantiateMsg};
    use sg_multi_test::StargazeApp;
    use sg_std::{StargazeMsgWrapper, GENESIS_MINT_START_TIME};
    use vending_factory::helpers::FactoryContract;
    use vending_factory::msg::{
        ExecuteMsg, InstantiateMsg as FactoryInstantiateMsg, VendingMinterCreateMsg,
        VendingMinterInitMsgExtension,
    };
    use vending_factory::state::{ParamsExtension, VendingMinterParams};

    pub fn factory_contract() -> Box<dyn Contract<StargazeMsgWrapper>> {
        let contract = ContractWrapper::new(
            vending_factory::contract::execute,
            vending_factory::contract::instantiate,
            vending_factory::contract::query,
        );
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

    pub fn sg721_base_contract() -> Box<dyn Contract<StargazeMsgWrapper>> {
        let contract = ContractWrapper::new(
            crate::entry::execute,
            crate::entry::instantiate,
            crate::entry::query,
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

    pub fn mock_init_extension() -> VendingMinterInitMsgExtension {
        VendingMinterInitMsgExtension {
            base_token_uri: "ipfs://aldkfjads".to_string(),
            payment_address: None,
            start_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME),
            num_tokens: 100,
            mint_price: coin(MIN_MINT_PRICE, NATIVE_DENOM),
            per_address_limit: 5,
            whitelist: None,
        }
    }

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
            max_trading_offset_secs: 60 * 60 * 24 * 7,
        }
    }

    pub fn mock_create_minter() -> VendingMinterCreateMsg {
        VendingMinterCreateMsg {
            init_msg: mock_init_extension(),
            collection_params: mock_collection_params(),
        }
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

    fn proper_instantiate() -> (StargazeApp, Addr) {
        let (mut app, factory_contract) = proper_instantiate_factory();
        let sg721_id = app.store_code(sg721_base_contract());

        let mut m = mock_create_minter();
        m.collection_params.code_id = sg721_id;
        let msg = ExecuteMsg::CreateMinter(m);

        let creation_fee = coin(CREATION_FEE, NATIVE_DENOM);

        app.sudo(SudoMsg::Bank(BankSudo::Mint {
            to_address: ADMIN.to_string(),
            amount: vec![creation_fee.clone()],
        }))
        .unwrap();

        let bal = app.wrap().query_all_balances(ADMIN).unwrap();
        assert_eq!(bal, vec![creation_fee.clone()]);

        // this should create the minter + sg721
        let cosmos_msg = factory_contract.call_with_funds(msg, creation_fee).unwrap();

        let res = app.execute(Addr::unchecked(ADMIN), cosmos_msg);
        dbg!("{:?}", &res);
        assert!(res.is_ok());

        (app, Addr::unchecked("contract2"))
    }

    mod init {

        use super::*;
        use crate::msg::QueryMsg;

        #[test]
        fn create_sg721_base_collection() {
            let (app, contract) = proper_instantiate();

            let res: NumTokensResponse = app
                .wrap()
                .query_wasm_smart(contract, &QueryMsg::NumTokens {})
                .unwrap();
            assert_eq!(res.count, 0);
        }

        #[test]
        fn check_ready_unauthorized() {
            let mut app = custom_mock_app();
            let sg721_id = app.store_code(sg721_base_contract());
            let msg = InstantiateMsg {
                name: "sg721".to_string(),
                symbol: "STARGAZE".to_string(),
                minter: ADMIN.to_string(),
                collection_info: CollectionInfo {
                    creator: ADMIN.to_string(),
                    description: "description".to_string(),
                    image: "description".to_string(),
                    external_link: None,
                    explicit_content: None,
                    start_trading_time: None,
                    royalty_info: None,
                },
            };
            let res = app.instantiate_contract(
                sg721_id,
                Addr::unchecked(GOVERNANCE),
                &msg,
                &[],
                "sg721-only",
                None,
            );
            // should not let create the contract.
            assert!(res.is_err());
        }

        #[test]
        fn check_ready_authorized() {
            let (_, _) = proper_instantiate();
        }
    }

    mod start_trading_time {
        use cosmwasm_std::{Decimal, Empty};
        use sg721::{RoyaltyInfoResponse, UpdateCollectionInfoMsg};

        use super::*;
        use crate::msg::{CollectionInfoResponse, QueryMsg};

        #[test]
        fn update_collection_info() {
            let params = mock_collection_params();
            let (app, contract) = proper_instantiate();

            // default trading start time is start time + default trading start time offset
            let res: CollectionInfoResponse = app
                .wrap()
                .query_wasm_smart(contract, &QueryMsg::CollectionInfo {})
                .unwrap();
            let default_start_time = mock_init_extension()
                .start_time
                .plus_seconds(mock_params().max_trading_offset_secs);
            assert_eq!(res.start_trading_time, Some(default_start_time));

            // update collection info
            let (mut app, contract) = proper_instantiate();

            let creator = Addr::unchecked("creator".to_string());

            // succeeds
            let res = app.execute_contract(
                creator.clone(),
                contract.clone(),
                &Sg721ExecuteMsg::<Empty, Empty>::UpdateCollectionInfo {
                    collection_info: UpdateCollectionInfoMsg {
                        description: Some(params.info.description.clone()),
                        image: Some(params.info.image.clone()),
                        external_link: Some(params.info.external_link.clone()),
                        explicit_content: None,
                        royalty_info: None,
                    },
                },
                &[],
            );
            assert!(res.is_ok());

            // update royalty_info
            let royalty_info: Option<RoyaltyInfoResponse> = Some(RoyaltyInfoResponse {
                payment_address: creator.to_string(),
                share: Decimal::percent(10),
            });
            let res = app.execute_contract(
                creator.clone(),
                contract.clone(),
                &Sg721ExecuteMsg::<Empty, Empty>::UpdateCollectionInfo {
                    collection_info: UpdateCollectionInfoMsg {
                        description: Some(params.info.description.clone()),
                        image: Some(params.info.image.clone()),
                        external_link: Some(params.info.external_link.clone()),
                        explicit_content: None,
                        royalty_info: Some(royalty_info.clone()),
                    },
                },
                &[],
            );
            assert!(res.is_ok());

            let res: CollectionInfoResponse = app
                .wrap()
                .query_wasm_smart(contract.clone(), &QueryMsg::CollectionInfo {})
                .unwrap();
            assert_eq!(res.royalty_info.unwrap(), royalty_info.clone().unwrap());

            // update explicit content
            let res = app.execute_contract(
                creator.clone(),
                contract.clone(),
                &Sg721ExecuteMsg::<Empty, Empty>::UpdateCollectionInfo {
                    collection_info: UpdateCollectionInfoMsg {
                        description: Some(params.info.description.clone()),
                        image: Some(params.info.image.clone()),
                        external_link: Some(params.info.external_link.clone()),
                        explicit_content: Some(true),
                        royalty_info: Some(royalty_info),
                    },
                },
                &[],
            );
            assert!(res.is_ok());

            let res: CollectionInfoResponse = app
                .wrap()
                .query_wasm_smart(contract.clone(), &QueryMsg::CollectionInfo {})
                .unwrap();
            // check explicit content changed to true
            assert!(res.explicit_content.unwrap());

            // try update royalty_info higher
            let royalty_info: Option<RoyaltyInfoResponse> = Some(RoyaltyInfoResponse {
                payment_address: creator.to_string(),
                share: Decimal::percent(11),
            });
            let res = app.execute_contract(
                creator.clone(),
                contract.clone(),
                &Sg721ExecuteMsg::<Empty, Empty>::UpdateCollectionInfo {
                    collection_info: UpdateCollectionInfoMsg {
                        description: None,
                        image: None,
                        external_link: None,
                        explicit_content: None,
                        royalty_info: Some(royalty_info),
                    },
                },
                &[],
            );
            assert!(res.is_err());

            // freeze collection throw err if not creator
            let res = app.execute_contract(
                Addr::unchecked("badguy"),
                contract.clone(),
                &Sg721ExecuteMsg::<Empty, Empty>::FreezeCollectionInfo {},
                &[],
            );
            assert!(res.is_err());
            // freeze collection to prevent further updates
            let res = app.execute_contract(
                creator.clone(),
                contract.clone(),
                &Sg721ExecuteMsg::<Empty, Empty>::FreezeCollectionInfo {},
                &[],
            );
            assert!(res.is_ok());

            // trying to update collection after frozen should throw err
            let res = app.execute_contract(
                creator,
                contract,
                &Sg721ExecuteMsg::<Empty, Empty>::UpdateCollectionInfo {
                    collection_info: UpdateCollectionInfoMsg {
                        description: Some(params.info.description.clone()),
                        image: Some(params.info.image.clone()),
                        external_link: Some(params.info.external_link),
                        explicit_content: None,
                        royalty_info: None,
                    },
                },
                &[],
            );
            assert!(res.is_err());
        }
    }
}
