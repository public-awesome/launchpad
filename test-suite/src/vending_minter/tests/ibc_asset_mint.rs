use crate::common_setup::{
    setup_collection_whitelist::WHITELIST_AMOUNT, setup_minter::common::constants::FOUNDATION,
};
use cosmwasm_std::{coin, coins, Addr, Decimal, Uint128};
use cw_multi_test::{BankSudo, Executor, SudoMsg};
use sg2::{msg::Sg2ExecuteMsg, tests::mock_collection_params};
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
use sg_whitelist::msg::{AddMembersMsg, ExecuteMsg as WhitelistExecuteMsg};
use vending_factory::ContractError;
use vending_minter::msg::ExecuteMsg;
use vending_minter::ContractError as MinterContractError;

use crate::common_setup::{
    contract_boxes::custom_mock_app,
    setup_accounts_and_block::{setup_accounts, setup_block_time},
    setup_collection_whitelist::setup_whitelist_contract,
    setup_minter::{
        common::constants::MINT_PRICE,
        vending_minter::{
            mock_params::{mock_create_minter_init_msg, mock_init_extension},
            setup::vending_minter_code_ids,
        },
    },
    templates::{vending_minter_template, vending_minter_with_ibc_asset},
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

    let mint_price = coins(MINT_PRICE, "ibc/asset".to_string());

    // give the buyer some of the IBC asset
    router
        .sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: buyer.to_string(),
                amount: mint_price.clone(),
            }
        }))
        .map_err(|err| println!("{err:?}"))
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
    let mut app = custom_mock_app();
    let (creator, _) = setup_accounts(&mut app);

    let mut init_msg = mock_init_extension(None, None);
    init_msg.mint_price = coin(MINT_PRICE, denom);

    let code_ids = vending_minter_code_ids(&mut app);

    let minter_code_id = code_ids.minter_code_id;
    let factory_code_id = code_ids.factory_code_id;
    let sg721_code_id = code_ids.sg721_code_id;
    let minter_admin = creator;

    let mut params = mock_params(None);
    params.code_id = minter_code_id;

    let factory_addr = app
        .instantiate_contract(
            factory_code_id,
            minter_admin.clone(),
            &vending_factory::msg::InstantiateMsg { params },
            &[],
            "factory",
            None,
        )
        .unwrap();

    let mut msg = mock_create_minter_init_msg(mock_collection_params(), init_msg);
    msg.collection_params.code_id = sg721_code_id;
    msg.collection_params.info.creator = minter_admin.to_string();
    let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);
    let msg = Sg2ExecuteMsg::CreateMinter(msg);

    let err = app
        .execute_contract(minter_admin, factory_addr, &msg, &creation_fee)
        .unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        ContractError::DenomMismatch {}.to_string()
    );
}

#[test]
fn wl_denom_mismatch() {
    // create factory and minter w NATIVE_DENOM, then try setting wl w different denom
    let num_tokens = 7000;
    let denom = "ibc/asset";
    let vt = vending_minter_template(num_tokens);
    let (mut router, creator, _) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();

    // setup whitelist with custom denom
    let whitelist_addr = setup_whitelist_contract(&mut router, &creator, None, Some(denom));

    // set whitelist in minter contract
    let set_whitelist_msg = ExecuteMsg::SetWhitelist {
        whitelist: whitelist_addr.to_string(),
    };
    let err = router
        .execute_contract(creator.clone(), minter_addr, &set_whitelist_msg, &[])
        .unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        MinterContractError::InvalidDenom {
            expected: NATIVE_DENOM.to_string(),
            got: denom.to_string(),
        }
        .to_string()
    );
}

#[test]
fn wl_denom_mint() {
    // create factory, minter, wl w custom denom, then try mint
    let denom = "ibc/asset";
    let mut app = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut app);

    let mut init_msg = mock_init_extension(None, None);
    init_msg.mint_price = coin(MINT_PRICE, denom);
    let code_ids = vending_minter_code_ids(&mut app);

    let minter_code_id = code_ids.minter_code_id;
    let factory_code_id = code_ids.factory_code_id;
    let sg721_code_id = code_ids.sg721_code_id;
    let minter_admin = creator.clone();

    let mut params = mock_params(Some(denom.to_string()));
    params.code_id = minter_code_id;

    let factory_addr = app
        .instantiate_contract(
            factory_code_id,
            minter_admin.clone(),
            &vending_factory::msg::InstantiateMsg { params },
            &[],
            "factory",
            None,
        )
        .unwrap();

    let mut msg = mock_create_minter_init_msg(mock_collection_params(), init_msg);
    msg.collection_params.code_id = sg721_code_id;
    msg.collection_params.info.creator = minter_admin.to_string();
    let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);
    let msg = Sg2ExecuteMsg::CreateMinter(msg);
    let res = app.execute_contract(minter_admin, factory_addr, &msg, &creation_fee);
    assert!(res.is_ok());
    let minter_addr = Addr::unchecked("contract1");

    // Try to set whitelist with different denom
    // setup whitelist with custom denom
    let different_denom = "ibc/otherdenom";
    let whitelist_addr = setup_whitelist_contract(&mut app, &creator, None, Some(different_denom));
    // add buyer to whitelist
    let add_to_whitelist_msg = WhitelistExecuteMsg::AddMembers(AddMembersMsg {
        to_add: vec![buyer.to_string()],
    });
    let res = app.execute_contract(
        creator.clone(),
        whitelist_addr.clone(),
        &add_to_whitelist_msg,
        &[],
    );
    assert!(res.is_ok());
    // set whitelist in minter contract
    let set_whitelist_msg = ExecuteMsg::SetWhitelist {
        whitelist: whitelist_addr.to_string(),
    };
    let res = app.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &set_whitelist_msg,
        &[],
    );
    assert!(res.is_err());

    // setup whitelist with custom denom
    let whitelist_addr = setup_whitelist_contract(&mut app, &creator, None, Some(denom));
    // add buyer to whitelist
    let add_to_whitelist_msg = WhitelistExecuteMsg::AddMembers(AddMembersMsg {
        to_add: vec![buyer.to_string()],
    });
    let res = app.execute_contract(
        creator.clone(),
        whitelist_addr.clone(),
        &add_to_whitelist_msg,
        &[],
    );
    assert!(res.is_ok());

    // set whitelist in minter contract
    let set_whitelist_msg = ExecuteMsg::SetWhitelist {
        whitelist: whitelist_addr.to_string(),
    };
    let res = app.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &set_whitelist_msg,
        &[],
    );
    assert!(res.is_ok());

    // give the buyer some of the IBC asset
    let wl_mint_price = coin(WHITELIST_AMOUNT, denom);
    app.sudo(SudoMsg::Bank({
        BankSudo::Mint {
            to_address: buyer.to_string(),
            amount: vec![wl_mint_price.clone()],
        }
    }))
    .map_err(|err| println!("{err:?}"))
    .ok();

    // set block time to whitelist start time
    setup_block_time(&mut app, GENESIS_MINT_START_TIME + 101, None);

    // Whitelist mint succeeds
    let mint_msg = ExecuteMsg::Mint {};
    let res = app.execute_contract(
        buyer.clone(),
        minter_addr,
        &mint_msg,
        &[wl_mint_price.clone()],
    );
    assert!(res.is_ok());

    // confirm balances
    // confirm buyer IBC assets spent
    let balance = app.wrap().query_balance(buyer, denom).unwrap();
    assert_eq!(balance.amount, Uint128::zero());
    // for seller should get 90% of IBC asset
    let balance = app.wrap().query_balance(creator, denom).unwrap();
    assert_eq!(balance.amount, wl_mint_price.amount * Decimal::percent(90));
    let balance = app
        .wrap()
        .query_balance(Addr::unchecked(FOUNDATION), denom)
        .unwrap();
    assert_eq!(balance.amount, wl_mint_price.amount * Decimal::percent(10));
}
