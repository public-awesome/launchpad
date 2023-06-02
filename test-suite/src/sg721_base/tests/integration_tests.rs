#[cfg(test)]
mod tests {
    use cosmwasm_std::{coin, Addr};
    use cw721::NumTokensResponse;
    use cw_multi_test::{BankSudo, Executor, SudoMsg};
    use sg2::msg::CreateMinterMsg;
    use sg2::tests::mock_collection_params;
    use sg721::ExecuteMsg as Sg721ExecuteMsg;
    use sg721::{CollectionInfo, InstantiateMsg};
    use sg_multi_test::StargazeApp;
    use vending_factory::helpers::FactoryContract;
    use vending_factory::msg::{
        ExecuteMsg, InstantiateMsg as FactoryInstantiateMsg, VendingMinterInitMsgExtension,
    };

    use crate::common_setup::contract_boxes::{
        contract_sg721_base, contract_vending_factory, contract_vending_minter, custom_mock_app,
    };
    use crate::common_setup::setup_minter::common::constants::CREATION_FEE;
    use crate::common_setup::setup_minter::vending_minter::mock_params::{
        mock_create_minter, mock_init_extension, mock_params,
    };

    const GOVERNANCE: &str = "governance";
    const ADMIN: &str = "admin";
    const NATIVE_DENOM: &str = "ustars";

    fn proper_instantiate_factory() -> (StargazeApp, FactoryContract) {
        let mut app = custom_mock_app();
        let factory_id = app.store_code(contract_vending_factory());
        let minter_id = app.store_code(contract_vending_minter());

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
        let sg721_id = app.store_code(contract_sg721_base());

        let collection_params = mock_collection_params();
        let mut m = mock_create_minter(None, collection_params, None);
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
        assert!(res.is_ok());

        (app, Addr::unchecked("contract2"))
    }

    fn custom_proper_instantiate(
        custom_create_minter_msg: CreateMinterMsg<VendingMinterInitMsgExtension>,
    ) -> (StargazeApp, Addr) {
        let (mut app, factory_contract) = proper_instantiate_factory();
        let sg721_id = app.store_code(contract_sg721_base());

        let mut m = custom_create_minter_msg;
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
        assert!(res.is_ok());

        (app, Addr::unchecked("contract2"))
    }

    mod init {

        use cw721_base::MinterResponse;

        use crate::common_setup::setup_minter::vending_minter::mock_params::mock_create_minter_init_msg;

        use super::*;
        use sg4::MinterConfigResponse;
        use sg721_base::msg::QueryMsg;
        use vending_minter::msg::QueryMsg as VendingMinterQueryMsg;
        use vending_minter::state::ConfigExtension as VendingMinterConfigExtension;

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
            let sg721_id = app.store_code(contract_sg721_base());
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
                    royalty_updated_at: None,
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

        #[test]
        fn sanitize_base_token_uri() {
            let base_token_uri = " ipfs://somecidhere ".to_string();
            let init_msg = VendingMinterInitMsgExtension {
                base_token_uri: base_token_uri.clone(),
                ..mock_init_extension(None, None)
            };
            let custom_create_minter_msg =
                mock_create_minter_init_msg(mock_collection_params(), init_msg);
            let (app, contract) = custom_proper_instantiate(custom_create_minter_msg);

            // query minter config to confirm base_token_uri got trimmed
            let res: MinterResponse = app
                .wrap()
                .query_wasm_smart(contract, &QueryMsg::Minter {})
                .unwrap();
            let minter = res.minter;
            let res: MinterConfigResponse<VendingMinterConfigExtension> = app
                .wrap()
                .query_wasm_smart(minter, &VendingMinterQueryMsg::Config {})
                .unwrap();
            assert_eq!(
                res.config.extension.base_token_uri,
                base_token_uri.trim().to_string()
            );

            // test sanitizing base token uri IPFS -> ipfs
            let base_token_uri = " IPFS://somecidhereipfs ".to_string();
            let init_msg = VendingMinterInitMsgExtension {
                base_token_uri,
                ..mock_init_extension(None, None)
            };
            let custom_create_minter_msg =
                mock_create_minter_init_msg(mock_collection_params(), init_msg);
            // let custom_create_minter_msg = mock_create_minter(None, mock_collection_params(), None);
            // let custom_create_minter_msg =
            //     custom_mock_create_minter(init_msg, mock_collection_params());

            let (app, contract) = custom_proper_instantiate(custom_create_minter_msg);

            // query minter config to confirm base_token_uri got trimmed and starts with ipfs
            let res: MinterResponse = app
                .wrap()
                .query_wasm_smart(contract, &QueryMsg::Minter {})
                .unwrap();
            let minter = res.minter;
            let res: MinterConfigResponse<VendingMinterConfigExtension> = app
                .wrap()
                .query_wasm_smart(minter, &VendingMinterQueryMsg::Config {})
                .unwrap();
            assert_eq!(
                res.config.extension.base_token_uri,
                "ipfs://somecidhereipfs"
            );

            // test case sensitive ipfs IPFS://aBcDeF -> ipfs://aBcDeF
            let base_token_uri = "IPFS://aBcDeF".to_string();
            let init_msg = VendingMinterInitMsgExtension {
                base_token_uri,
                ..mock_init_extension(None, None)
            };
            let custom_create_minter_msg =
                mock_create_minter_init_msg(mock_collection_params(), init_msg);

            let (app, contract) = custom_proper_instantiate(custom_create_minter_msg);
            let res: MinterResponse = app
                .wrap()
                .query_wasm_smart(contract, &QueryMsg::Minter {})
                .unwrap();
            let minter = res.minter;
            let res: MinterConfigResponse<VendingMinterConfigExtension> = app
                .wrap()
                .query_wasm_smart(minter, &VendingMinterQueryMsg::Config {})
                .unwrap();
            assert_eq!(res.config.extension.base_token_uri, "ipfs://aBcDeF");
        }
    }

    mod start_trading_time {
        use cosmwasm_std::{Decimal, Empty};
        use sg721::{RoyaltyInfoResponse, UpdateCollectionInfoMsg};

        use crate::common_setup::setup_minter::vending_minter::mock_params::mock_create_minter_init_msg;

        use super::*;
        use sg721_base::msg::{CollectionInfoResponse, QueryMsg};

        #[test]
        fn royalty_updates() {
            let mut params = mock_collection_params();
            params.info.external_link = None;
            params.info.royalty_info = None;
            let custom_create_minter_msg =
                mock_create_minter_init_msg(params, mock_init_extension(None, None));
            let (mut app, contract) = custom_proper_instantiate(custom_create_minter_msg);
            let creator = Addr::unchecked("creator".to_string());

            let royalty_info: Option<RoyaltyInfoResponse> = Some(RoyaltyInfoResponse {
                payment_address: creator.to_string(),
                share: Decimal::percent(11),
            });
            let res = app.execute_contract(
                creator,
                contract,
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
        }

        #[test]
        fn update_collection_info() {
            // customize params so external_link is None
            let mut params = mock_collection_params();
            params.info.external_link = None;
            let custom_create_minter_msg =
                mock_create_minter_init_msg(params.clone(), mock_init_extension(None, None));
            let (mut app, contract) = custom_proper_instantiate(custom_create_minter_msg.clone());

            // default trading start time is start time + default trading start time offset
            let res: CollectionInfoResponse = app
                .wrap()
                .query_wasm_smart(contract.clone(), &QueryMsg::CollectionInfo {})
                .unwrap();
            let default_start_time = mock_init_extension(None, None)
                .start_time
                .plus_seconds(mock_params().max_trading_offset_secs);
            assert_eq!(res.start_trading_time, Some(default_start_time));

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

    mod royalty_payout {
        use super::*;

        use crate::common_setup::setup_minter::vending_minter::mock_params::mock_create_minter_init_msg;
        use cosmwasm_std::{Decimal, Response, Uint128};
        use sg2::{msg::CollectionParams, tests::mock_collection_info};
        use sg721::RoyaltyInfoResponse;
        use sg721_base::msg::{CollectionInfoResponse, QueryMsg};

        #[test]
        fn standard_payout() {
            let (app, contract) = proper_instantiate();

            let res: CollectionInfoResponse = app
                .wrap()
                .query_wasm_smart(contract.clone(), &QueryMsg::CollectionInfo {})
                .unwrap();

            // payout 100stars, royalty share 10%, royalty payout 10stars
            let payment = Uint128::from(100000000u128);
            let royalty_share = res.clone().royalty_info.unwrap().share;
            let royalty_payout = res
                .royalty_payout(
                    contract,
                    payment,
                    Uint128::from(10000000u128),
                    None,
                    &mut Response::default(),
                )
                .unwrap();
            assert_eq!(royalty_payout, payment * royalty_share);
        }

        #[test]
        fn payout_0_royalties() {
            let init_msg = mock_init_extension(None, None);
            let custom_collection_params = CollectionParams {
                info: CollectionInfo {
                    creator: "creator".to_string(),
                    description: String::from("Stargaze Monkeys"),
                    image: "https://example.com/image.png".to_string(),
                    external_link: Some("https://example.com/external.html".to_string()),
                    start_trading_time: None,
                    explicit_content: Some(false),
                    royalty_info: Some(RoyaltyInfoResponse {
                        payment_address: "creator".to_string(),
                        share: Decimal::percent(0),
                    }),
                    royalty_updated_at: None,
                },
                ..mock_collection_params()
            };
            let custom_create_minter_msg =
                mock_create_minter_init_msg(custom_collection_params, init_msg);
            let (app, contract) = custom_proper_instantiate(custom_create_minter_msg);

            let res: CollectionInfoResponse = app
                .wrap()
                .query_wasm_smart(contract.clone(), &QueryMsg::CollectionInfo {})
                .unwrap();

            // payout 100stars, royalty share 0%, royalty payout 0stars
            let payment = Uint128::from(100000000u128);
            let royalty_payout = res
                .royalty_payout(
                    contract,
                    payment,
                    Uint128::from(10000000u128),
                    None,
                    &mut Response::default(),
                )
                .unwrap();
            assert_eq!(royalty_payout, Uint128::zero());
        }

        #[test]
        fn payout_too_much_royalties() {
            let init_msg = mock_init_extension(None, None);
            let custom_collection_params = CollectionParams {
                info: CollectionInfo {
                    creator: "creator".to_string(),
                    description: String::from("Stargaze Monkeys"),
                    image: "https://example.com/image.png".to_string(),
                    external_link: Some("https://example.com/external.html".to_string()),
                    start_trading_time: None,
                    explicit_content: Some(false),
                    royalty_info: Some(RoyaltyInfoResponse {
                        payment_address: "creator".to_string(),
                        share: Decimal::percent(91),
                    }),
                    royalty_updated_at: None,
                },
                ..mock_collection_params()
            };
            let custom_create_minter_msg =
                mock_create_minter_init_msg(custom_collection_params, init_msg);
            let (app, contract) = custom_proper_instantiate(custom_create_minter_msg);

            let res: CollectionInfoResponse = app
                .wrap()
                .query_wasm_smart(contract.clone(), &QueryMsg::CollectionInfo {})
                .unwrap();

            // payout 100stars, royalty share 91%, royalty payout fails
            // fees exceed payment
            let payment = Uint128::from(100000000u128);
            let res = res.royalty_payout(
                contract,
                payment,
                Uint128::from(10000000u128),
                None,
                &mut Response::default(),
            );
            assert!(res.is_err());
        }

        #[test]
        fn payout_odd_royalties() {
            // uint * decimal::percent
            let init_msg = mock_init_extension(None, None);
            let custom_collection_params = CollectionParams {
                info: CollectionInfo {
                    creator: "creator".to_string(),
                    description: String::from("Stargaze Monkeys"),
                    image: "https://example.com/image.png".to_string(),
                    external_link: Some("https://example.com/external.html".to_string()),
                    start_trading_time: None,
                    explicit_content: Some(false),
                    royalty_info: Some(RoyaltyInfoResponse {
                        payment_address: "creator".to_string(),
                        share: Decimal::percent(3),
                    }),
                    royalty_updated_at: None,
                },
                ..mock_collection_params()
            };
            let custom_create_minter_msg =
                mock_create_minter_init_msg(custom_collection_params, init_msg);
            let (app, contract) = custom_proper_instantiate(custom_create_minter_msg);

            let res: CollectionInfoResponse = app
                .wrap()
                .query_wasm_smart(contract.clone(), &QueryMsg::CollectionInfo {})
                .unwrap();

            // payout 100stars, royalty share 1%, royalty payout 10stars
            let payment = Uint128::from(1111111111121111111u128);
            let royalty_share = res.clone().royalty_info.unwrap().share;
            let royalty_payout = res
                .royalty_payout(
                    contract,
                    payment,
                    Uint128::from(10000000u128),
                    None,
                    &mut Response::default(),
                )
                .unwrap();
            assert_eq!(royalty_payout, payment * royalty_share);
        }

        #[test]
        fn payout_royalties_none() {
            let init_msg = mock_init_extension(None, None);
            let custom_collection_params = CollectionParams {
                info: CollectionInfo {
                    creator: "creator".to_string(),
                    description: String::from("Stargaze Monkeys"),
                    image: "https://example.com/image.png".to_string(),
                    external_link: Some("https://example.com/external.html".to_string()),
                    start_trading_time: None,
                    explicit_content: Some(false),
                    royalty_info: None,
                    ..mock_collection_info()
                },
                ..mock_collection_params()
            };
            let custom_create_minter_msg =
                mock_create_minter_init_msg(custom_collection_params, init_msg);
            let (app, contract) = custom_proper_instantiate(custom_create_minter_msg);

            let res: CollectionInfoResponse = app
                .wrap()
                .query_wasm_smart(contract.clone(), &QueryMsg::CollectionInfo {})
                .unwrap();

            // payout 100stars, royalty share none, royalty payout 0stars
            let payment = Uint128::from(100000000u128);
            let royalty_payout = res
                .royalty_payout(
                    contract,
                    payment,
                    Uint128::from(10000000u128),
                    None,
                    &mut Response::default(),
                )
                .unwrap();
            assert_eq!(royalty_payout, Uint128::zero());
        }
    }
}
