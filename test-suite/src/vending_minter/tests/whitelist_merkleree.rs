use cosmwasm_std::{coin, coins, Empty, Timestamp};
use cw721::{Cw721QueryMsg, TokensResponse};
use cw721_base::ExecuteMsg as Cw721ExecuteMsg;
use cw_multi_test::Executor;
use sg2::tests::mock_collection_params_1;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};

use crate::common_setup::msg::MinterCollectionResponse;
use crate::common_setup::setup_accounts_and_block::coins_for_msg;
use crate::common_setup::setup_whitelist_merkletree::setup_whitelist_mtree_contract;
use crate::common_setup::setup_minter::common::minter_params::minter_params_token;
use crate::common_setup::setup_minter::vending_minter::setup::{
    configure_minter, vending_minter_code_ids,
};
use crate::common_setup::{
    contract_boxes::custom_mock_app,
    setup_accounts_and_block::{setup_accounts, setup_block_time},
};
use vending_minter::msg::{MintCountResponse, QueryMsg, ExecuteMsg};
use vending_minter::ContractError;

use whitelist_mtree::{
    tests::test_helpers::tree_from_vec,
    msg::ExecuteMsg as WhitelistExecuteMsg
};

const MINT_PRICE: u128 = 100_000_000;
const WHITELIST_AMOUNT: u128 = 66_000_000;


#[test]
fn works_with_regular_whitelist() {
    let mut router = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut router);
    let num_tokens = 10;
    let start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    let collection_params = mock_collection_params_1(Some(start_time));
    let minter_params = minter_params_token(num_tokens);
    let code_ids = vending_minter_code_ids(&mut router);
    let minter_collection_response: Vec<MinterCollectionResponse> = configure_minter(
        &mut router,
        creator.clone(),
        vec![collection_params],
        vec![minter_params],
        code_ids,
    );
    let minter_addr = minter_collection_response[0].minter.clone().unwrap();
    let collection_addr = minter_collection_response[0].collection.clone().unwrap();

    let addresses = vec![
        buyer.to_string(),
        "another_address".to_string(),
    ];

    let tree = tree_from_vec(&addresses);
    let root = tree.root_hex().unwrap();

    let whitelist_addr = setup_whitelist_mtree_contract(&mut router, &creator, None, None, root);
    const EXPIRATION_TIME: Timestamp = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10_000);


    let set_whitelist_msg = vending_minter::msg::ExecuteMsg::SetWhitelist {
        whitelist: whitelist_addr.to_string(),
    };
    let res = router.execute_contract(creator.clone(), minter_addr.clone(), &set_whitelist_msg, &[]);
    assert!(res.is_ok());

    // Set block to before genesis mint start time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME - 1000, None);

    let wl_msg = WhitelistExecuteMsg::UpdateEndTime(EXPIRATION_TIME);
    let res = router.execute_contract(creator.clone(), whitelist_addr.clone(), &wl_msg, &[]);
    assert!(res.is_ok());

    let wl_msg = WhitelistExecuteMsg::UpdateStartTime(Timestamp::from_nanos(0));
    let res = router.execute_contract(creator.clone(), whitelist_addr.clone(), &wl_msg, &[]);
    assert!(res.is_ok());


    setup_block_time(&mut router, GENESIS_MINT_START_TIME, Some(10));

    // Can't mint without proofs
    let mint_msg = ExecuteMsg::Mint { proof_hashes: None };
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(WHITELIST_AMOUNT, NATIVE_DENOM),
    );
    assert!(res.is_err());

    let proof_hashes = tree.proof(&[0]).proof_hashes_hex();

    let mint_msg = ExecuteMsg::Mint { proof_hashes: Some(proof_hashes.clone()) };
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
    let mint_msg = ExecuteMsg::Mint { proof_hashes: Some(proof_hashes) };
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
    let mint_msg = ExecuteMsg::Mint { proof_hashes: None };
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
        .query_wasm_smart(collection_addr.clone(), &tokens_msg)
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
        collection_addr,
        &transfer_msg,
        &coins_for_msg(coin(123, NATIVE_DENOM)),
    );
    assert!(res.is_ok());

    // Mint succeeds
    let mint_msg = ExecuteMsg::Mint { proof_hashes: None };
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
    let mint_msg = ExecuteMsg::Mint { proof_hashes: None };
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

