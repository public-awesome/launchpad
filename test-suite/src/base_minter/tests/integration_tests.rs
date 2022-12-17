// use core::num;

// use crate::common_setup::{
//     contract_boxes::{
//         contract_base_factory, contract_base_minter, contract_factory, contract_minter,
//         contract_nt_collection, custom_mock_app,
//     },
//     msg::MinterSetupParams,
//     setup_accounts_and_block::setup_accounts,
//     setup_minter::minter_params_token,
// };
// use base_factory::msg::BaseMinterCreateMsg;
// use base_factory::state::BaseMinterParams;
// use base_minter::msg::{ConfigResponse, ExecuteMsg};
// use cosmwasm_std::{coin, coins, Addr, Timestamp};
// use cw721::{Cw721ExecuteMsg, Cw721QueryMsg, OwnerOfResponse};
// use cw_multi_test::Executor;
// use sg2::tests::mock_collection_params;
// use sg2::{msg::Sg2ExecuteMsg, tests::mock_collection_params_1};
// use sg4::QueryMsg;
// use sg721_base::msg::{CollectionInfoResponse, QueryMsg as Sg721QueryMsg};
// use sg_multi_test::StargazeApp;
// use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};

// const CREATION_FEE: u128 = 1_000_000_000;
// pub const MIN_MINT_PRICE: u128 = 50_000_000;
// const MINT_FEE_BPS: u64 = 10_000; // 100%

// pub fn mock_params() -> BaseMinterParams {
//     BaseMinterParams {
//         code_id: 1,
//         creation_fee: coin(CREATION_FEE, NATIVE_DENOM),
//         min_mint_price: coin(MIN_MINT_PRICE, NATIVE_DENOM),
//         mint_fee_bps: MINT_FEE_BPS,
//         max_trading_offset_secs: 60 * 60 * 24 * 7,
//         extension: None,
//     }
// }

// pub fn mock_create_minter() -> BaseMinterCreateMsg {
//     BaseMinterCreateMsg {
//         init_msg: None,
//         collection_params: mock_collection_params(),
//     }
// }

// Upload contract code and instantiate minter contract
// fn setup_minter_contract(
//     router: &mut StargazeApp,
//     creator: &Addr,
//     nt_collection: bool,
// ) -> (Addr, ConfigResponse) {
//     let minter_code_id = router.store_code(contract_base_minter());
//     let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);

//     let factory_code_id = router.store_code(contract_base_factory());

//     let mut params = mock_params();
//     params.code_id = minter_code_id;

//     let factory_addr = router
//         .instantiate_contract(
//             factory_code_id,
//             creator.clone(),
//             &base_factory::msg::InstantiateMsg { params },
//             &[],
//             "factory",
//             None,
//         )
//         .unwrap();

//     let collection_code_id = if nt_collection {
//         router.store_code(contract_nt_collection())
//     } else {
//         router.store_code(contract_nt_collection())
//     };

//     let mut msg = mock_create_minter();
//     msg.collection_params.code_id = collection_code_id;
//     msg.collection_params.info.creator = creator.to_string();

//     let msg = Sg2ExecuteMsg::CreateMinter(msg);

//     let balances = router.wrap().query_all_balances(creator.clone()).unwrap();
//     assert_eq!(balances, coins(INITIAL_BALANCE, NATIVE_DENOM));

//     let res = router.execute_contract(creator.clone(), factory_addr, &msg, &creation_fee);
//     assert!(res.is_ok());

//     let balances = router.wrap().query_all_balances(creator.clone()).unwrap();
//     assert_eq!(
//         balances,
//         coins(INITIAL_BALANCE - CREATION_FEE, NATIVE_DENOM)
//     );

//     // could get the minter address from the response above, but we know its contract1
//     let minter_addr = Addr::unchecked("contract1");

//     let config: ConfigResponse = router
//         .wrap()
//         .query_wasm_smart(minter_addr.clone(), &QueryMsg::Config {})
//         .unwrap();

//     (minter_addr, config)
// }

// #[test]
// fn check_mint() {
//     // let mut router = custom_mock_app();
//     // let (creator, buyer) = setup_accounts(&mut router);
//     // // let (minter_addr, config) = setup_minter_contract(&mut router, &creator, true);

//     let mut router = custom_mock_app();
//     let (creator, buyer) = setup_accounts(&mut router);
//     let start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
//     let num_tokens = 1;
//     let collection_params = mock_collection_params_1(Some(start_time));
//     let minter_params = minter_params_token(num_tokens);

//     let factory_code_id = router.store_code(contract_factory());
//     let minter_code_id = router.store_code(contract_base_minter());
//     let sg721_code_id = router.store_code(contract_nt_collection());
//     let minter_setup_params: MinterSetupParams = MinterSetupParams {
//         router: &mut router,
//         minter_admin: creator.clone(),
//         num_tokens: num_tokens,
//         collection_params: collection_params,
//         splits_addr: None,
//         factory_code_id,
//         minter_code_id,
//         sg721_code_id,
//         start_time: Some(start_time),
//         init_msg: None,
//     };
// }
//     let minter_collection_response = setup_minter_contract(minter_setup_params);
//     let minter_addr = minter_collection_response.minter.unwrap();
//     let collection_addr = minter_collection_response.collection.unwrap();

//     // Fail with incorrect token uri
//     let mint_msg = ExecuteMsg::Mint {
//         token_uri: "test uri".to_string(),
//     };
//     let err = router.execute_contract(creator.clone(), minter_addr.clone(), &mint_msg, &[]);
//     assert!(err.is_err());

//     // Fail with incorrect mint price
//     let mint_msg = ExecuteMsg::Mint {
//         token_uri: "ipfs://example".to_string(),
//     };
//     let err = router.execute_contract(
//         creator.clone(),
//         minter_addr.clone(),
//         &mint_msg,
//         &[coin(MIN_MINT_PRICE + 100, NATIVE_DENOM)],
//     );
//     assert!(err.is_err());

//     // Not authorized to mint
//     let mint_msg = ExecuteMsg::Mint {
//         token_uri: "ipfs://example".to_string(),
//     };
//     let err = router.execute_contract(
//         buyer,
//         minter_addr.clone(),
//         &mint_msg,
//         &[coin(MIN_MINT_PRICE, NATIVE_DENOM)],
//     );
//     assert!(err.is_err());

//     // Succeeds if funds are sent
//     let mint_msg = ExecuteMsg::Mint {
//         token_uri: "ipfs://example".to_string(),
//     };
//     let res = router.execute_contract(
//         creator.clone(),
//         minter_addr.clone(),
//         &mint_msg,
//         &[coin(MIN_MINT_PRICE, NATIVE_DENOM)],
//     );

//     println!("res is {:?}", res);
//     // assert!(res.is_ok());

//     // let creator_balances = router.wrap().query_all_balances(creator.clone()).unwrap();
//     // // assert_eq!(
//     // //     creator_balances,
//     // //     coins(
//     // //         INITIAL_BALANCE - CREATION_FEE - MIN_MINT_PRICE,
//     // //         NATIVE_DENOM
//     // //     )
//     // // );

//     // let res: ConfigResponse = router
//     //     .wrap()
//     //     .query_wasm_smart(minter_addr, &QueryMsg::Config {})
//     //     .unwrap();
//     // assert_eq!(res.collection_address, "contract2".to_string());
//     // assert_eq!(res.config.mint_price.amount.u128(), MIN_MINT_PRICE);

//     // let query_owner_msg = Cw721QueryMsg::OwnerOf {
//     //     token_id: String::from("1"),
//     //     include_expired: None,
//     // };
//     // let res: OwnerOfResponse = router
//     //     .wrap()
//     //     .query_wasm_smart(collection_addr.clone(), &query_owner_msg)
//     //     .unwrap();
//     // assert_eq!(res.owner, creator.to_string());

//     // // make sure sg721-nt cannot be transferred
//     // let transfer_msg = Cw721ExecuteMsg::TransferNft {
//     //     recipient: "adsf".to_string(),
//     //     token_id: "1".to_string(),
//     // };
//     // let err = router.execute_contract(
//     //     creator.clone(),
//     //     Addr::unchecked(collection_addr),
//     //     &transfer_msg,
//     //     &[],
//     // );
//     // assert!(err.is_err());
// }

// // #[test]
// // fn update_start_trading_time() {
// //     let mut router = custom_mock_app();
// //     let (creator, buyer) = setup_accounts(&mut router);
// //     let current_block_time = router.block_info().time;
// //     let (minter_addr, config) = setup_minter_contract(&mut router, &creator, false);
// //     let default_start_trading_time =
// //         current_block_time.plus_seconds(mock_params().max_trading_offset_secs + 1);

// //     // unauthorized
// //     let res = router.execute_contract(
// //         Addr::unchecked(buyer),
// //         Addr::unchecked(minter_addr.clone()),
// //         &ExecuteMsg::UpdateStartTradingTime(Some(default_start_trading_time)),
// //         &[],
// //     );
// //     assert!(res.is_err());

// //     // invalid start trading time
// //     let res = router.execute_contract(
// //         Addr::unchecked(creator.clone()),
// //         Addr::unchecked(minter_addr.clone()),
// //         &ExecuteMsg::UpdateStartTradingTime(Some(Timestamp::from_nanos(0))),
// //         &[],
// //     );
// //     assert!(res.is_err());

// //     // succeeds
// //     let res = router.execute_contract(
// //         Addr::unchecked(creator.clone()),
// //         Addr::unchecked(minter_addr),
// //         &ExecuteMsg::UpdateStartTradingTime(Some(default_start_trading_time)),
// //         &[],
// //     );
// //     assert!(res.is_ok());

// //     // confirm trading start time
// //     let res: CollectionInfoResponse = router
// //         .wrap()
// //         .query_wasm_smart(config.collection_address, &Sg721QueryMsg::CollectionInfo {})
// //         .unwrap();
// //     assert_eq!(res.start_trading_time, Some(default_start_trading_time));
// // }
