use cosmwasm_std::{coin, coins, Uint128};
use cw_multi_test::{BankSudo, Executor, SudoMsg};
use open_edition_factory::types::{NftData, NftMetadataType};
use open_edition_minter::msg::ExecuteMsg;
use sg_std::GENESIS_MINT_START_TIME;

use crate::common_setup::{
    contract_boxes::{contract_sg721_base, custom_mock_app},
    setup_accounts_and_block::{setup_accounts, setup_block_time},
    setup_minter::{
        common::constants::MIN_MINT_PRICE_OPEN_EDITION,
        open_edition_minter::{
            mock_params::mock_init_minter_extension, setup::open_edition_minter_code_ids,
        },
    },
    templates::open_edition_minter_custom_template,
};

#[test]
fn check_custom_create_minter_denom() {
    // allow ibc/frenz denom
    let denom = "ibc/frenz";
    let mint_price = coins(MIN_MINT_PRICE_OPEN_EDITION, denom.to_string());
    let vt = open_edition_minter_custom_template(
        None,
        None,
        None,
        Some(10),
        Some(2),
        Some(mint_price[0]),
        Some(denom),
        None,
        None,
    )
    .unwrap();
    let (mut router, _, buyer) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();

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
    let mut app = custom_mock_app();
    let (creator, _) = setup_accounts(&mut app);

    let mint_price = coin(MIN_MINT_PRICE_OPEN_EDITION, denom.to_string());
    let nft_data = NftData {
        nft_data_type: NftMetadataType::OffChainMetadata,
        token_uri: None,
        extension: None,
    };
    let mut init_msg =
        mock_init_minter_extension(None, None, None, Some(mint_price), nft_data, None);
    init_msg.mint_price = mint_price;

    let code_ids = open_edition_minter_code_ids(&mut app, contract_sg721_base());

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

// TODO add wl denom mismatch test

// TODO add wl mint test
