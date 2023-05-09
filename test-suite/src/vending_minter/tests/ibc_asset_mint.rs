use cosmwasm_std::{coin, coins, Timestamp};
use cw_multi_test::{BankSudo, Executor, SudoMsg};
use sg2::{
    msg::Sg2ExecuteMsg,
    tests::{mock_collection_params, mock_collection_params_1},
};
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
use vending_factory::ContractError;
use vending_minter::msg::ExecuteMsg;

use crate::common_setup::{
    contract_boxes::custom_mock_app,
    msg::MinterSetupParams,
    setup_accounts_and_block::{setup_accounts, setup_block_time},
    setup_minter::{
        common::{constants::MINT_PRICE, minter_params::minter_params_all},
        vending_minter::{
            mock_params::{mock_create_minter_init_msg, mock_init_extension},
            setup::vending_minter_code_ids,
        },
    },
    templates::vending_minter_with_ibc_asset,
};

use crate::common_setup::setup_minter::common::constants::CREATION_FEE;
use crate::common_setup::setup_minter::vending_minter::mock_params::mock_params;

#[test]
fn mint_with_ibc_asset() {
    let num_tokens = 7000;
    let per_address_limit = 10;
    let denom = "ibc/asset";
    let vt = vending_minter_with_ibc_asset(num_tokens, per_address_limit, denom);
    let (mut router, _, buyer) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();

    let mint_price = coins(MINT_PRICE, denom.to_string());

    // give the buyer some of the IBC asset
    router
        .sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: buyer.to_string(),
                amount: mint_price.clone(),
            }
        }))
        .map_err(|err| println!("{:?}", err))
        .ok();

    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 1, None);

    // Mint succeeds
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(buyer, minter_addr, &mint_msg, &mint_price);
    assert!(res.is_ok());
}

#[test]
fn denom_mismatch_creating_minter() {
    // create factory w NATIVE_DENOM, then try creating a minter w different denom
    let denom = "ibc/asset";
    let num_tokens = 2;
    let mut app = custom_mock_app();
    let (creator, _) = setup_accounts(&mut app);
    let start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    let collection_params = mock_collection_params_1(Some(start_time));

    let init_msg = vending_factory::msg::VendingMinterInitMsgExtension {
        base_token_uri: "ipfs://aldkfjads".to_string(),
        payment_address: None,
        start_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME),
        num_tokens,
        mint_price: coin(MINT_PRICE, denom),
        per_address_limit: 1,
        whitelist: Some("invalid address".to_string()),
    };

    let minter_params = minter_params_all(num_tokens, None, None, Some(init_msg));
    let code_ids = vending_minter_code_ids(&mut app);

    let setup_params: MinterSetupParams = MinterSetupParams {
        router: &mut app,
        minter_admin: creator,
        num_tokens,
        collection_params,
        splits_addr: minter_params.splits_addr,
        minter_code_id: code_ids.minter_code_id,
        factory_code_id: code_ids.factory_code_id,
        sg721_code_id: code_ids.sg721_code_id,
        start_time: minter_params.start_time,
        init_msg: minter_params.init_msg,
    };

    let minter_code_id = setup_params.minter_code_id;
    let router = setup_params.router;
    let factory_code_id = setup_params.factory_code_id;
    let sg721_code_id = setup_params.sg721_code_id;
    let minter_admin = setup_params.minter_admin;

    let mut params = mock_params(None);
    params.code_id = minter_code_id;

    let factory_addr = router
        .instantiate_contract(
            factory_code_id,
            minter_admin.clone(),
            &vending_factory::msg::InstantiateMsg { params },
            &[],
            "factory",
            None,
        )
        .unwrap();

    let mut init_msg = mock_init_extension(None, None);
    init_msg.mint_price = coin(MINT_PRICE, denom);
    let mut msg = mock_create_minter_init_msg(mock_collection_params(), init_msg);
    msg.collection_params.code_id = sg721_code_id;
    msg.collection_params.info.creator = minter_admin.to_string();
    let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);
    let msg = Sg2ExecuteMsg::CreateMinter(msg);

    let err = router
        .execute_contract(minter_admin, factory_addr, &msg, &creation_fee)
        .unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        ContractError::DenomMismatch {}.to_string()
    );
}

// #[test]
// fn wl_mint_price() {
//     let num_tokens = 2;
//     let mut app = custom_mock_app();
//     let (creator, buyer) = setup_accounts(&mut app);
//     let start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
//     let collection_params = mock_collection_params_1(Some(start_time));

//     let init_msg = vending_factory::msg::VendingMinterInitMsgExtension {
//         base_token_uri: "ipfs://aldkfjads".to_string(),
//         payment_address: None,
//         start_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME),
//         num_tokens,
//         mint_price: coin(MINT_PRICE, NATIVE_DENOM),
//         per_address_limit: 1,
//         whitelist: Some("invalid address".to_string()),
//     };

//     let minter_params = minter_params_all(num_tokens, None, None, Some(init_msg));
//     let code_ids = vending_minter_code_ids(&mut app);

//     let setup_params: MinterSetupParams = MinterSetupParams {
//         router: &mut app,
//         minter_admin: creator.clone(),
//         num_tokens,
//         collection_params,
//         splits_addr: minter_params.splits_addr,
//         minter_code_id: code_ids.minter_code_id,
//         factory_code_id: code_ids.factory_code_id,
//         sg721_code_id: code_ids.sg721_code_id,
//         start_time: minter_params.start_time,
//         init_msg: minter_params.init_msg,
//     };

//     let minter_code_id = setup_params.minter_code_id;
//     let router = setup_params.router;
//     let factory_code_id = setup_params.factory_code_id;
//     let sg721_code_id = setup_params.sg721_code_id;
//     let minter_admin = setup_params.minter_admin;

//     let mut params = mock_params(None);
//     params.code_id = minter_code_id;
//     params.min_mint_price = coin(MINT_PRICE, NATIVE_DENOM);

//     let factory_addr = router
//         .instantiate_contract(
//             factory_code_id,
//             minter_admin.clone(),
//             &vending_factory::msg::InstantiateMsg { params },
//             &[],
//             "factory",
//             None,
//         )
//         .unwrap();

//     let mut init_msg = mock_init_extension(None, None);
//     init_msg.mint_price = coin(MINT_PRICE, NATIVE_DENOM);
//     let mut msg = mock_create_minter_init_msg(mock_collection_params(), init_msg);
//     msg.collection_params.code_id = sg721_code_id;
//     msg.collection_params.info.creator = minter_admin.to_string();
//     let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);
//     let msg = Sg2ExecuteMsg::CreateMinter(msg);

//     let res = router.execute_contract(minter_admin, factory_addr, &msg, &creation_fee);
//     assert!(res.is_ok());

//     let minter_addr = Addr::unchecked("contract1");

//     // set up free mint whitelist
//     let whitelist_addr = setup_zero_fee_whitelist_contract(router, &creator, None);
//     let msg = ExecuteMsg::UpdateStartTime(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 1000));
//     router
//         .execute_contract(creator.clone(), minter_addr.clone(), &msg, &[])
//         .unwrap();
//     // set whitelist on minter
//     let msg = ExecuteMsg::SetWhitelist {
//         whitelist: whitelist_addr.to_string(),
//     };
//     router
//         .execute_contract(creator.clone(), minter_addr.clone(), &msg, &[])
//         .unwrap();
//     // add buyer to whitelist
//     let msg = sg_whitelist::msg::ExecuteMsg::AddMembers(AddMembersMsg {
//         to_add: vec![buyer.to_string()],
//     });
//     router
//         .execute_contract(creator.clone(), whitelist_addr, &msg, &[])
//         .unwrap();
//     setup_block_time(router, GENESIS_MINT_START_TIME + 100, None);

//     // mint succeeds
//     let mint_msg = ExecuteMsg::Mint {};
//     let res = router.execute_contract(buyer, minter_addr, &mint_msg, &[]);
//     assert!(res.is_ok());
// }
