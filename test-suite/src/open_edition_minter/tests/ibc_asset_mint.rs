// use base_factory::ContractError as BaseContractError;
// use cosmwasm_std::{coin, coins, Addr, Decimal, Uint128};
// use cw_multi_test::{BankSudo, Executor, SudoMsg};
// use open_edition_factory::types::{NftData, NftMetadataType};
// use open_edition_minter::msg::ExecuteMsg;
// use sg2::{msg::Sg2ExecuteMsg, tests::mock_collection_params};
// use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};

// use crate::common_setup::{
//     contract_boxes::{contract_sg721_base, custom_mock_app},
//     setup_accounts_and_block::{setup_accounts, setup_block_time},
//     setup_minter::{
//         common::constants::{CREATION_FEE, MIN_MINT_PRICE_OPEN_EDITION},
//         open_edition_minter::{
//             mock_params::{
//                 mock_create_minter_init_msg, mock_init_minter_extension, mock_params_custom,
//             },
//             setup::open_edition_minter_code_ids,
//         },
//     },
//     templates::{open_edition_minter_custom_template, OpenEditionMinterCustomParams},
// };

// #[test]
// fn check_custom_create_minter_denom() {
//     // allow ibc/frenz denom
//     let denom = "ibc/frenz";
//     let mint_price = coin(MIN_MINT_PRICE_OPEN_EDITION, denom.to_string());
//     let custom_params = OpenEditionMinterCustomParams {
//         denom: Some(denom),
//         ..OpenEditionMinterCustomParams::default()
//     };
//     let vt = open_edition_minter_custom_template(
//         None,
//         None,
//         None,
//         Some(10),
//         Some(2),
//         Some(mint_price.clone()),
//         custom_params,
//         None,
//         None,
//     )
//     .unwrap();
//     let (mut router, creator, buyer) = (vt.router, vt.accts.creator, vt.accts.buyer);
//     let minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();

//     // give the buyer some of the IBC asset
//     router
//         .sudo(SudoMsg::Bank({
//             BankSudo::Mint {
//                 to_address: buyer.to_string(),
//                 amount: vec![mint_price.clone()],
//             }
//         }))
//         .map_err(|err| println!("{err:?}"))
//         .ok();

//     setup_block_time(&mut router, GENESIS_MINT_START_TIME + 100, None);

//     // Mint succeeds
//     let mint_msg = ExecuteMsg::Mint {};
//     let res = router.execute_contract(buyer.clone(), minter_addr, &mint_msg, &[mint_price.clone()]);
//     assert!(res.is_ok());

//     // confirm balances
//     // confirm buyer IBC assets spent
//     let balance = router.wrap().query_balance(buyer, denom).unwrap();
//     assert_eq!(balance.amount, Uint128::zero());
//     // TODO only for noble, seller has 90% IBC asset
//     let network_fee = mint_price.amount * Decimal::percent(10);
//     let seller_amount = mint_price.amount.checked_sub(network_fee).unwrap();
//     let balance = router.wrap().query_balance(creator, denom).unwrap();
//     assert_eq!(balance.amount, seller_amount);
//     // all mint goes to fairburn_pool confirmed in e2e test
// }

// #[test]
// fn one_hundred_percent_burned_ibc_minter() {
//     // factory needs airdrop_mint_price: 0
//     // factory needs mint_fee_bps: 100_00 (100%)
//     // 100% fairburn, so 50% goes to dev, 50% goes to community pool

//     // allow ibc/frenz denom
//     let denom = "ibc/frenz";
//     let mint_price = coin(MIN_MINT_PRICE_OPEN_EDITION, denom.to_string());
//     let custom_params = OpenEditionMinterCustomParams {
//         denom: Some(denom),
//         mint_fee_bps: Some(10000),
//         airdrop_mint_price_amount: Some(Uint128::zero()),
//     };
//     let vt = open_edition_minter_custom_template(
//         None,
//         None,
//         None,
//         Some(10),
//         Some(2),
//         Some(mint_price.clone()),
//         custom_params,
//         None,
//         None,
//     )
//     .unwrap();
//     let (mut router, creator, buyer) = (vt.router, vt.accts.creator, vt.accts.buyer);
//     let minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();

//     // give the buyer some of the IBC asset
//     router
//         .sudo(SudoMsg::Bank({
//             BankSudo::Mint {
//                 to_address: buyer.to_string(),
//                 amount: vec![mint_price.clone()],
//             }
//         }))
//         .map_err(|err| println!("{err:?}"))
//         .ok();

//     setup_block_time(&mut router, GENESIS_MINT_START_TIME + 100, None);

//     // Mint succeeds
//     let mint_msg = ExecuteMsg::Mint {};
//     let res = router.execute_contract(buyer.clone(), minter_addr, &mint_msg, &[mint_price.clone()]);
//     assert!(res.is_ok());

//     // confirm balances
//     // confirm buyer IBC assets spent
//     let balance = router.wrap().query_balance(buyer, denom).unwrap();
//     assert_eq!(balance.amount, Uint128::zero());
//     // for noble, seller has 0% IBC asset
//     let balance = router.wrap().query_balance(creator, denom).unwrap();
//     assert_eq!(balance.amount, Uint128::zero());
//     // confirm mint_price 50% sent to community pool, 50% sent to dev
//     // "community_pool" address from packages/sg-multi-test/src/multi.rs
//     let balance = router
//         .wrap()
//         .query_balance(Addr::unchecked("community_pool"), denom)
//         .unwrap();
//     assert_eq!(balance.amount, mint_price.amount * Decimal::percent(50));
// }

// #[test]
// fn zero_mint_fee() {
//     // factory needs airdrop_mint_price: 0
//     // factory needs mint_fee_bps: 0 (0%)

//     // allow ibc/frenz denom
//     let denom = "ibc/frenz";
//     let mint_price = coin(MIN_MINT_PRICE_OPEN_EDITION, denom.to_string());
//     let custom_params = OpenEditionMinterCustomParams {
//         denom: Some(denom),
//         mint_fee_bps: Some(0),
//         airdrop_mint_price_amount: Some(Uint128::zero()),
//     };
//     let vt = open_edition_minter_custom_template(
//         None,
//         None,
//         None,
//         Some(10),
//         Some(2),
//         Some(mint_price.clone()),
//         custom_params,
//         None,
//         None,
//     )
//     .unwrap();
//     let (mut router, _, buyer) = (vt.router, vt.accts.creator, vt.accts.buyer);
//     let minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();

//     // give the buyer some of the IBC asset
//     router
//         .sudo(SudoMsg::Bank({
//             BankSudo::Mint {
//                 to_address: buyer.to_string(),
//                 amount: vec![mint_price.clone()],
//             }
//         }))
//         .map_err(|err| println!("{err:?}"))
//         .ok();

//     setup_block_time(&mut router, GENESIS_MINT_START_TIME + 100, None);

//     // Mint succeeds
//     let mint_msg = ExecuteMsg::Mint {};
//     let res = router.execute_contract(buyer, minter_addr, &mint_msg, &[mint_price]);
//     assert!(res.is_ok());
// }

// #[test]
// fn denom_mismatch_creating_minter() {
//     // create factory w NATIVE_DENOM, then try creating a minter w different denom
//     let denom = "ibc/asset";
//     let mut app = custom_mock_app();
//     let (creator, _) = setup_accounts(&mut app);

//     let mint_price = coin(MIN_MINT_PRICE_OPEN_EDITION, denom.to_string());
//     let nft_data = NftData {
//         nft_data_type: NftMetadataType::OffChainMetadata,
//         token_uri: Some("ipfs://1234".to_string()),
//         extension: None,
//     };
//     let code_ids = open_edition_minter_code_ids(&mut app, contract_sg721_base());

//     let minter_code_id = code_ids.minter_code_id;
//     let factory_code_id = code_ids.factory_code_id;
//     let sg721_code_id = code_ids.sg721_code_id;
//     let minter_admin = creator;

//     let mut params = mock_params_custom(OpenEditionMinterCustomParams::default());
//     params.code_id = minter_code_id;

//     let factory_addr = app
//         .instantiate_contract(
//             factory_code_id,
//             minter_admin.clone(),
//             &open_edition_factory::msg::InstantiateMsg { params },
//             &[],
//             "factory",
//             None,
//         )
//         .unwrap();

//     let mut init_msg =
//         mock_init_minter_extension(None, None, None, Some(mint_price.clone()), nft_data, None);
//     init_msg.mint_price = mint_price;
//     let mut msg = mock_create_minter_init_msg(mock_collection_params(), init_msg);
//     msg.collection_params.code_id = sg721_code_id;
//     msg.collection_params.info.creator = minter_admin.to_string();
//     let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);
//     let msg = Sg2ExecuteMsg::CreateMinter(msg);

//     let err = app
//         .execute_contract(minter_admin, factory_addr, &msg, &creation_fee)
//         .unwrap_err();
//     assert_eq!(
//         err.source().unwrap().to_string(),
//         BaseContractError::InvalidDenom {}.to_string()
//     );
// }
