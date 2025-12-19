use crate::common_setup::contract_boxes::contract_sg721_base;
use crate::common_setup::setup_accounts_and_block::{CREATION_FEE, INITIAL_BALANCE};
use crate::common_setup::setup_minter::base_minter::mock_params::mock_params;
use crate::common_setup::setup_minter::common::constants::MIN_MINT_PRICE;
use crate::common_setup::templates::{
    base_minter_with_sg721, base_minter_with_sg721nt, base_minter_with_specified_sg721,
};
use base_factory::msg::{BaseMinterCreateMsg, BaseUpdateParamsMsg, SudoMsg};

use base_minter::msg::{ConfigResponse, ExecuteMsg};
use cosmwasm_std::{coin, coins, Addr, Timestamp};
use cw721::msg::{CollectionInfoAndExtensionResponse, OwnerOfResponse};
use cw721::DefaultOptionalCollectionExtension;
use cw721_base::msg::{ExecuteMsg as Cw721ExecuteMsg, QueryMsg as Cw721QueryMsg};
type Sg721QueryMsg = Cw721QueryMsg;
use cw_multi_test::Executor;
use sg2::msg::Sg2ExecuteMsg;
use sg2::query::{AllowedCollectionCodeIdsResponse, Sg2QueryMsg};
use sg2::tests::mock_collection_params_1;
use sg4::QueryMsg;
use sg_utils::{FEE_DENOM, GENESIS_MINT_START_TIME, NATIVE_DENOM};

#[test]
fn init() {
    let bmt = base_minter_with_sg721nt(1);
    bmt.collection_response_vec[0].minter.clone().unwrap();
    bmt.collection_response_vec[0].collection.clone().unwrap();
}

#[test]
fn update_code_id() {
    let sg721_code_id = 7u64;
    let bmt = base_minter_with_specified_sg721(1, sg721_code_id);
    let (mut router, creator, _) = (bmt.router, bmt.accts.creator, bmt.accts.buyer);
    let factory = bmt.collection_response_vec[0].factory.clone().unwrap();

    // sg721 code id not in allowed code ids
    let res = bmt.collection_response_vec[0].minter.clone();
    assert!(res.is_none());

    // add sg721_code_id to allowed code ids
    let update_msg = BaseUpdateParamsMsg {
        add_sg721_code_ids: Some(vec![sg721_code_id]),
        rm_sg721_code_ids: None,
        frozen: None,
        code_id: None,
        creation_fee: None,
        min_mint_price: None,
        mint_fee_bps: None,
        max_trading_offset_secs: None,
        extension: None,
    };
    let sudo_msg = SudoMsg::UpdateParams(Box::new(update_msg));
    let res = router.wasm_sudo(factory.clone(), &sudo_msg);
    assert!(res.is_ok());

    let msg = Sg2QueryMsg::AllowedCollectionCodeIds {};
    let res: AllowedCollectionCodeIdsResponse = router
        .wrap()
        .query_wasm_smart(factory.clone(), &msg)
        .unwrap();
    assert!(res.code_ids.contains(&sg721_code_id));

    // store sg721_base 4-7 code ids
    for _ in 0..(sg721_code_id - 3) {
        router.store_code(contract_sg721_base());
    }

    // create minter with sg721_code_id
    let start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    let mut collection_params = mock_collection_params_1(Some(start_time));
    collection_params.code_id = sg721_code_id;

    let mut msg = BaseMinterCreateMsg {
        init_msg: None,
        collection_params,
    };
    msg.collection_params.creator = creator.to_string();
    let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);
    let msg = Sg2ExecuteMsg::CreateMinter(msg);
    let res = router.execute_contract(creator, factory, &msg, &creation_fee);
    assert!(res.is_ok());

    // Extract collection address from the response events
    // There are two instantiate events: minter (code_id 1) and collection (code_id = sg721_code_id)
    // We need the collection address which is the second instantiate event
    let res = res.unwrap();
    let instantiate_events: Vec<_> = res
        .events
        .iter()
        .filter(|e| e.ty == "instantiate")
        .collect();

    // The collection is the second instantiate event (after the minter)
    let collection_addr = instantiate_events
        .get(1)
        .and_then(|e| {
            e.attributes
                .iter()
                .find(|a| a.key == "_contract_address" || a.key == "_contract_addr")
                .map(|a| a.value.clone())
        })
        .expect("Collection address not found in events");

    // confirm new sg721 code id == sg721_code_id
    let res = router
        .wrap()
        .query_wasm_contract_info(collection_addr)
        .unwrap();
    assert!(res.code_id == sg721_code_id);
}

#[test]
fn check_mint() {
    let bmt = base_minter_with_sg721nt(1);
    let (mut router, creator, buyer) = (bmt.router, bmt.accts.creator, bmt.accts.buyer);
    let minter_addr = bmt.collection_response_vec[0].minter.clone().unwrap();
    let collection_addr = bmt.collection_response_vec[0].collection.clone().unwrap();

    // Fail with incorrect token uri
    let mint_msg = ExecuteMsg::Mint {
        token_uri: "test uri".to_string(),
    };
    let err = router.execute_contract(creator.clone(), minter_addr.clone(), &mint_msg, &[]);
    assert!(err.is_err());

    // Fail with incorrect mint price
    let mint_msg = ExecuteMsg::Mint {
        token_uri: "ipfs://example".to_string(),
    };
    let err = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &mint_msg,
        &[coin(MIN_MINT_PRICE + 100, FEE_DENOM)],
    );
    assert!(err.is_err());

    // Not authorized to mint
    let mint_msg = ExecuteMsg::Mint {
        token_uri: "ipfs://example".to_string(),
    };
    let err = router.execute_contract(
        buyer,
        minter_addr.clone(),
        &mint_msg,
        &[coin(MIN_MINT_PRICE, FEE_DENOM)],
    );
    assert!(err.is_err());

    // Succeeds if funds are sent
    let mint_msg = ExecuteMsg::Mint {
        token_uri: "ipfs://example".to_string(),
    };
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &mint_msg,
        &[coin(MIN_MINT_PRICE, FEE_DENOM)],
    );

    println!("res is {res:?}");
    assert!(res.is_ok());

    // Check FEE_DENOM balance (used for minting)
    let creator_fee_balance = router
        .wrap()
        .query_balance(creator.clone(), FEE_DENOM)
        .unwrap();
    assert_eq!(
        creator_fee_balance.amount.u128(),
        (INITIAL_BALANCE + CREATION_FEE) - MIN_MINT_PRICE
    );

    let res: ConfigResponse = router
        .wrap()
        .query_wasm_smart(minter_addr, &QueryMsg::Config {})
        .unwrap();
    // With MockApiBech32, collection_address is a proper bech32 address
    assert_eq!(res.collection_address, collection_addr.to_string());
    assert_eq!(res.config.mint_price.amount.u128(), MIN_MINT_PRICE);

    let query_owner_msg = Cw721QueryMsg::OwnerOf {
        token_id: String::from("1"),
        include_expired: None,
    };
    let res: OwnerOfResponse = router
        .wrap()
        .query_wasm_smart(collection_addr.clone(), &query_owner_msg)
        .unwrap();
    assert_eq!(res.owner, creator.to_string());

    // make sure sg721-nt cannot be transferred
    let transfer_msg = Cw721ExecuteMsg::TransferNft {
        recipient: "adsf".to_string(),
        token_id: "1".to_string(),
    };
    let err = router.execute_contract(
        creator,
        Addr::unchecked(collection_addr),
        &transfer_msg,
        &[],
    );
    assert!(err.is_err());
}

#[test]
fn update_start_trading_time() {
    let bmt = base_minter_with_sg721(1);
    let (mut router, creator, buyer) = (bmt.router, bmt.accts.creator, bmt.accts.buyer);
    let minter_addr = bmt.collection_response_vec[0].minter.clone().unwrap();
    let collection_addr = bmt.collection_response_vec[0].collection.clone().unwrap();
    let current_block_time = router.block_info().time;

    let default_start_trading_time =
        current_block_time.plus_seconds(mock_params().max_trading_offset_secs + 1);

    // unauthorized
    let res = router.execute_contract(
        Addr::unchecked(buyer),
        Addr::unchecked(minter_addr.clone()),
        &ExecuteMsg::UpdateStartTradingTime(Some(default_start_trading_time)),
        &[],
    );
    assert!(res.is_err());

    // invalid start trading time
    let res = router.execute_contract(
        Addr::unchecked(creator.clone()),
        Addr::unchecked(minter_addr.clone()),
        &ExecuteMsg::UpdateStartTradingTime(Some(Timestamp::from_nanos(0))),
        &[],
    );
    assert!(res.is_err());

    // succeeds
    let res = router.execute_contract(
        Addr::unchecked(creator),
        Addr::unchecked(minter_addr),
        &ExecuteMsg::UpdateStartTradingTime(Some(default_start_trading_time)),
        &[],
    );
    println!("res is {res:?}");
    assert!(res.is_ok());

    // confirm trading start time
    let res: CollectionInfoAndExtensionResponse<DefaultOptionalCollectionExtension> = router
        .wrap()
        .query_wasm_smart(
            collection_addr,
            &Sg721QueryMsg::GetCollectionInfoAndExtension {},
        )
        .unwrap();
    assert_eq!(
        res.extension.as_ref().and_then(|e| e.start_trading_time),
        Some(default_start_trading_time)
    );
}
