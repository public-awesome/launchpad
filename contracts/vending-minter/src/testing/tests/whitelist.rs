use cosmwasm_std::{coin, coins, Addr, Empty, Timestamp};
use cw721::{Cw721QueryMsg, TokensResponse};
use cw721_base::ExecuteMsg as Cw721ExecuteMsg;
use cw_multi_test::Executor;
use sg2::msg::Sg2ExecuteMsg;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};

use crate::msg::{ExecuteMsg, MintCountResponse, MintPriceResponse, QueryMsg};
use crate::testing::setup::setup_accounts_and_block::coins_for_msg;
use crate::testing::setup::{
    setup_accounts_and_block::{setup_accounts, setup_block_time},
    setup_contracts::{
        contract_factory, contract_minter, contract_sg721, custom_mock_app, mock_create_minter,
        mock_params, setup_minter_contract, setup_whitelist_contract,
    },
};
use crate::ContractError;

use sg_whitelist::msg::{
    AddMembersMsg, ConfigResponse as WhitelistConfigResponse, ExecuteMsg as WhitelistExecuteMsg,
    QueryMsg as WhitelistQueryMsg,
};

const CREATION_FEE: u128 = 5_000_000_000;
const MINT_PRICE: u128 = 100_000_000;
const WHITELIST_AMOUNT: u128 = 66_000_000;

#[test]
fn invalid_whitelist_instantiate() {
    let mut router = custom_mock_app();
    let (creator, _) = setup_accounts(&mut router);

    let num_tokens = 10;
    let minter_code_id = router.store_code(contract_minter());
    let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);
    let factory_code_id = router.store_code(contract_factory());

    let mut params = mock_params();
    params.code_id = minter_code_id;

    let factory_addr = router
        .instantiate_contract(
            factory_code_id,
            creator.clone(),
            &vending_factory::msg::InstantiateMsg { params },
            &[],
            "factory",
            None,
        )
        .unwrap();

    let sg721_code_id = router.store_code(contract_sg721());

    let mut msg = mock_create_minter(None);
    msg.init_msg.whitelist = Some("invalid address".to_string());
    msg.init_msg.mint_price = coin(MINT_PRICE, NATIVE_DENOM);
    msg.init_msg.num_tokens = num_tokens;
    msg.collection_params.code_id = sg721_code_id;
    msg.collection_params.info.creator = creator.to_string();

    let msg = Sg2ExecuteMsg::CreateMinter(msg);

    let err = router
        .execute_contract(creator, factory_addr, &msg, &creation_fee)
        .unwrap_err();
    assert_eq!(
        err.source().unwrap().source().unwrap().to_string(),
        "Generic error: Querier contract error: cw_multi_test::wasm::ContractData not found"
    );
}

#[test]
fn set_invalid_whitelist() {
    let mut router = custom_mock_app();
    let (creator, _) = setup_accounts(&mut router);
    let num_tokens = 10;
    let (minter_addr, _) = setup_minter_contract(&mut router, &creator, num_tokens, None);
    let whitelist_addr = setup_whitelist_contract(&mut router, &creator);
    const EXPIRATION_TIME: Timestamp = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10_000);

    // Set block to before genesis mint start time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME - 1000, None);

    // Update to a start time after genesis
    let msg = ExecuteMsg::UpdateStartTime(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 1000));
    router
        .execute_contract(creator.clone(), minter_addr.clone(), &msg, &[])
        .unwrap();

    // update wl times
    const WL_START: Timestamp = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 200);

    let wl_msg = WhitelistExecuteMsg::UpdateStartTime(WL_START);
    let res = router.execute_contract(creator.clone(), whitelist_addr.clone(), &wl_msg, &[]);
    assert!(res.is_ok());
    let wl_msg = WhitelistExecuteMsg::UpdateEndTime(EXPIRATION_TIME);
    let res = router.execute_contract(creator.clone(), whitelist_addr.clone(), &wl_msg, &[]);
    assert!(res.is_ok());

    // Set whitelist in minter contract
    let set_whitelist_msg = ExecuteMsg::SetWhitelist {
        whitelist: "invalid".to_string(),
    };
    let err = router
        .execute_contract(
            creator.clone(),
            minter_addr.clone(),
            &set_whitelist_msg,
            &[],
        )
        .unwrap_err();
    assert_eq!(
        err.source().unwrap().source().unwrap().to_string(),
        "Generic error: Querier contract error: cw_multi_test::wasm::ContractData not found"
    );

    // move time to make wl start
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 201, Some(11));

    // check that the new whitelist exists
    let wl_res: WhitelistConfigResponse = router
        .wrap()
        .query_wasm_smart(whitelist_addr.to_string(), &WhitelistQueryMsg::Config {})
        .unwrap();

    assert!(wl_res.is_active);

    // Set whitelist in minter contract
    let set_whitelist_msg = ExecuteMsg::SetWhitelist {
        whitelist: whitelist_addr.to_string(),
    };

    let err = router
        .execute_contract(creator.clone(), minter_addr, &set_whitelist_msg, &[])
        .unwrap_err();
    assert_eq!(err.source().unwrap().to_string(), "WhitelistAlreadyStarted");
}

#[test]
fn whitelist_mint_count_query() {
    let mut router = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut router);
    let num_tokens = 10;
    let (minter_addr, config) = setup_minter_contract(&mut router, &creator, num_tokens, None);
    let sg721_addr = Addr::unchecked(config.sg721_address);
    let whitelist_addr = setup_whitelist_contract(&mut router, &creator);
    const EXPIRATION_TIME: Timestamp = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10_000);

    // Set block to before genesis mint start time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME - 1000, None);

    let wl_msg = WhitelistExecuteMsg::UpdateEndTime(EXPIRATION_TIME);
    let res = router.execute_contract(creator.clone(), whitelist_addr.clone(), &wl_msg, &[]);
    assert!(res.is_ok());

    let wl_msg = WhitelistExecuteMsg::UpdateStartTime(Timestamp::from_nanos(0));
    let res = router.execute_contract(creator.clone(), whitelist_addr.clone(), &wl_msg, &[]);
    assert!(res.is_ok());

    // Set whitelist in minter contract
    let set_whitelist_msg = ExecuteMsg::SetWhitelist {
        whitelist: whitelist_addr.to_string(),
    };
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &set_whitelist_msg,
        &[],
    );
    assert!(res.is_ok());

    // Update per address_limit
    let set_whitelist_msg = ExecuteMsg::UpdatePerAddressLimit {
        per_address_limit: 3,
    };
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &set_whitelist_msg,
        &[],
    );
    assert!(res.is_ok());

    // Add buyer to whitelist
    let inner_msg = AddMembersMsg {
        to_add: vec![buyer.to_string()],
    };
    let wasm_msg = WhitelistExecuteMsg::AddMembers(inner_msg);
    let res = router.execute_contract(creator.clone(), whitelist_addr, &wasm_msg, &[]);
    assert!(res.is_ok());

    setup_block_time(&mut router, GENESIS_MINT_START_TIME, Some(10));

    // Mint succeeds
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(WHITELIST_AMOUNT, NATIVE_DENOM),
    );
    assert!(res.is_ok());

    // Query count
    let res: MintCountResponse = router
        .wrap()
        .query_wasm_smart(
            minter_addr.clone(),
            &QueryMsg::MintCount {
                address: buyer.to_string(),
            },
        )
        .unwrap();
    assert_eq!(res.count, 1);
    assert_eq!(res.address, buyer.to_string());

    // Mint fails, over whitelist per address limit
    let mint_msg = ExecuteMsg::Mint {};
    let err = router
        .execute_contract(
            buyer.clone(),
            minter_addr.clone(),
            &mint_msg,
            &coins(WHITELIST_AMOUNT, NATIVE_DENOM),
        )
        .unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        ContractError::MaxPerAddressLimitExceeded {}.to_string()
    );

    // Set time after wl ends
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 20_000, Some(11));

    // Public mint succeeds
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_ok());

    // Query count
    let res: MintCountResponse = router
        .wrap()
        .query_wasm_smart(
            minter_addr.clone(),
            &QueryMsg::MintCount {
                address: buyer.to_string(),
            },
        )
        .unwrap();
    assert_eq!(res.count, 2);
    assert_eq!(res.address, buyer.to_string());

    // get random mint token_id
    let tokens_msg = Cw721QueryMsg::Tokens {
        owner: buyer.to_string(),
        start_after: None,
        limit: None,
    };
    let res: TokensResponse = router
        .wrap()
        .query_wasm_smart(sg721_addr.clone(), &tokens_msg)
        .unwrap();
    let sold_token_id: u32 = res.tokens[1].parse::<u32>().unwrap();
    // Buyer transfers NFT to creator
    // random mint token id: 8
    let transfer_msg: Cw721ExecuteMsg<Empty, Empty> = Cw721ExecuteMsg::TransferNft {
        recipient: creator.to_string(),
        // token_id: "8".to_string(),
        token_id: sold_token_id.to_string(),
    };
    let res = router.execute_contract(
        buyer.clone(),
        sg721_addr,
        &transfer_msg,
        &coins_for_msg(coin(123, NATIVE_DENOM)),
    );
    assert!(res.is_ok());

    // Mint succeeds
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_ok());

    // Query count
    let res: MintCountResponse = router
        .wrap()
        .query_wasm_smart(
            minter_addr.clone(),
            &QueryMsg::MintCount {
                address: buyer.to_string(),
            },
        )
        .unwrap();
    assert_eq!(res.count, 3);
    assert_eq!(res.address, buyer.to_string());

    // Mint fails
    let mint_msg = ExecuteMsg::Mint {};
    let err = router
        .execute_contract(
            buyer.clone(),
            minter_addr.clone(),
            &mint_msg,
            &coins(WHITELIST_AMOUNT, NATIVE_DENOM),
        )
        .unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        ContractError::MaxPerAddressLimitExceeded {}.to_string()
    );

    // Query count
    let res: MintCountResponse = router
        .wrap()
        .query_wasm_smart(
            minter_addr,
            &QueryMsg::MintCount {
                address: buyer.to_string(),
            },
        )
        .unwrap();
    assert_eq!(res.count, 3);
    assert_eq!(res.address, buyer.to_string());
}

#[test]
fn whitelist_already_started() {
    let mut router = custom_mock_app();
    let (creator, _) = setup_accounts(&mut router);
    let num_tokens = 1;
    let (minter_addr, _) = setup_minter_contract(&mut router, &creator, num_tokens, None);
    let whitelist_addr = setup_whitelist_contract(&mut router, &creator);

    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 101, None);

    // set whitelist in minter contract
    let set_whitelist_msg = ExecuteMsg::SetWhitelist {
        whitelist: whitelist_addr.to_string(),
    };
    router
        .execute_contract(
            creator.clone(),
            minter_addr,
            &set_whitelist_msg,
            &coins(MINT_PRICE, NATIVE_DENOM),
        )
        .unwrap_err();
}

#[test]
fn whitelist_can_update_before_start() {
    let mut router = custom_mock_app();
    let (creator, _) = setup_accounts(&mut router);
    let num_tokens = 1;
    let (minter_addr, _) = setup_minter_contract(&mut router, &creator, num_tokens, None);
    let whitelist_addr = setup_whitelist_contract(&mut router, &creator);

    setup_block_time(&mut router, GENESIS_MINT_START_TIME - 1000, None);

    // set whitelist in minter contract
    let set_whitelist_msg = ExecuteMsg::SetWhitelist {
        whitelist: whitelist_addr.to_string(),
    };
    router
        .execute_contract(
            creator.clone(),
            minter_addr.clone(),
            &set_whitelist_msg,
            &[],
        )
        .unwrap();

    // can set twice before starting
    router
        .execute_contract(creator.clone(), minter_addr, &set_whitelist_msg, &[])
        .unwrap();
}

#[test]
fn whitelist_access_len_add_remove_expiration() {
    let mut router = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut router);
    let num_tokens = 1;
    let (minter_addr, config) = setup_minter_contract(&mut router, &creator, num_tokens, None);
    let sg721_addr = config.sg721_address;
    let whitelist_addr = setup_whitelist_contract(&mut router, &creator);
    const AFTER_GENESIS_TIME: Timestamp = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 100);

    // Set to just before genesis mint start time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME - 10, None);

    // Update whitelist_expiration fails if not admin
    let wl_msg = WhitelistExecuteMsg::UpdateEndTime(AFTER_GENESIS_TIME);
    router
        .execute_contract(buyer.clone(), whitelist_addr.clone(), &wl_msg, &[])
        .unwrap_err();

    // Update whitelist_expiration succeeds when from admin
    let wl_msg = WhitelistExecuteMsg::UpdateEndTime(AFTER_GENESIS_TIME);
    let res = router.execute_contract(creator.clone(), whitelist_addr.clone(), &wl_msg, &[]);
    assert!(res.is_ok());

    let wl_msg = WhitelistExecuteMsg::UpdateStartTime(Timestamp::from_nanos(0));
    let res = router.execute_contract(creator.clone(), whitelist_addr.clone(), &wl_msg, &[]);
    assert!(res.is_ok());

    // Set whitelist in minter contract
    let set_whitelist_msg = ExecuteMsg::SetWhitelist {
        whitelist: whitelist_addr.to_string(),
    };
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &set_whitelist_msg,
        &[],
    );
    assert!(res.is_ok());

    // Mint fails, buyer is not on whitelist
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_err());

    // Add buyer to whitelist
    let inner_msg = AddMembersMsg {
        to_add: vec![buyer.to_string()],
    };
    let wasm_msg = WhitelistExecuteMsg::AddMembers(inner_msg);
    let res = router.execute_contract(creator.clone(), whitelist_addr.clone(), &wasm_msg, &[]);
    assert!(res.is_ok());

    // Mint fails, not whitelist price
    let mint_msg = ExecuteMsg::Mint {};
    router
        .execute_contract(
            buyer.clone(),
            minter_addr.clone(),
            &mint_msg,
            &coins(MINT_PRICE, NATIVE_DENOM),
        )
        .unwrap_err();

    setup_block_time(&mut router, GENESIS_MINT_START_TIME, None);

    // Query mint price
    let mint_price_response: MintPriceResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &QueryMsg::MintPrice {})
        .unwrap();

    assert_eq!(
        coin(WHITELIST_AMOUNT, NATIVE_DENOM),
        mint_price_response.whitelist_price.unwrap()
    );
    assert_eq!(
        coin(WHITELIST_AMOUNT, NATIVE_DENOM),
        mint_price_response.current_price
    );
    assert_eq!(
        coin(MINT_PRICE, NATIVE_DENOM),
        mint_price_response.public_price
    );

    // Mint succeeds with whitelist price
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(WHITELIST_AMOUNT, NATIVE_DENOM),
    );
    assert!(res.is_ok());

    // Mint fails, over whitelist per address limit
    let mint_msg = ExecuteMsg::Mint {};
    let err = router
        .execute_contract(
            buyer.clone(),
            minter_addr.clone(),
            &mint_msg,
            &coins(WHITELIST_AMOUNT, NATIVE_DENOM),
        )
        .unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        ContractError::MaxPerAddressLimitExceeded {}.to_string()
    );

    // Muyer is generous and transfers to creator
    let transfer_msg: Cw721ExecuteMsg<Empty, Empty> = Cw721ExecuteMsg::TransferNft {
        recipient: creator.to_string(),
        token_id: "1".to_string(),
    };
    let res = router.execute_contract(
        buyer.clone(),
        Addr::unchecked(sg721_addr),
        &transfer_msg,
        &coins_for_msg(coin(123, NATIVE_DENOM)),
    );
    assert!(res.is_ok());

    // Mint fails, buyer exceeded per address limit
    let mint_msg = ExecuteMsg::Mint {};
    let err = router
        .execute_contract(
            buyer.clone(),
            minter_addr.clone(),
            &mint_msg,
            &coins(WHITELIST_AMOUNT, NATIVE_DENOM),
        )
        .unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        ContractError::MaxPerAddressLimitExceeded {}.to_string()
    );

    // Remove buyer from whitelist
    let inner_msg = AddMembersMsg { to_add: vec![] };
    let wasm_msg = WhitelistExecuteMsg::AddMembers(inner_msg);
    let res = router.execute_contract(creator.clone(), whitelist_addr, &wasm_msg, &[]);
    assert!(res.is_ok());

    // Mint fails
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer,
        minter_addr,
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_err());
}
