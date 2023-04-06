use cosmwasm_std::{coins, Empty};
use cw721::{Cw721QueryMsg, NftInfoResponse, TokensResponse};
use cw721_base::Extension;
use cw_multi_test::Executor;
use sg721_updatable::msg::ExecuteMsg as Sg721UpdatableExecMsg;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
use vending_minter::msg::ExecuteMsg;

use crate::common_setup::{
    setup_accounts_and_block::setup_block_time, setup_minter::common::constants::MINT_PRICE,
    templates::vending_minter_with_sg721_updatable,
};

#[test]
fn update_token_metadata() {
    // create updatable collection and minter
    let vt = vending_minter_with_sg721_updatable(10);
    let (mut router, creator, buyer) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let collection_addr = vt.collection_response_vec[0].collection.clone().unwrap();
    let minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();

    // Set block forward, after start time. mint succeeds
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 10_000_000, None);

    // mint token
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr,
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_ok());

    // query buyer token_id
    let query_tokens_msg = Cw721QueryMsg::Tokens {
        owner: buyer.to_string(),
        start_after: None,
        limit: None,
    };
    let res: TokensResponse = router
        .wrap()
        .query_wasm_smart(collection_addr.clone(), &query_tokens_msg)
        .unwrap();
    let token_id = res.tokens[0].to_string();

    // update token metadata
    let token_uri = Some("ipfs://new_token_uri".to_string());
    let msg = Sg721UpdatableExecMsg::<Empty, Empty>::UpdateTokenMetadata {
        token_id: token_id.clone(),
        token_uri: token_uri.clone(),
    };
    let res = router.execute_contract(creator, collection_addr.clone(), &msg, &[]);
    assert!(res.is_ok());

    // check token metadata
    let res: TokensResponse = router
        .wrap()
        .query_wasm_smart(collection_addr.clone(), &query_tokens_msg)
        .unwrap();
    assert_eq!(res.tokens[0], token_id);
    let query_token_msg = Cw721QueryMsg::NftInfo { token_id };
    let res: NftInfoResponse<Extension> = router
        .wrap()
        .query_wasm_smart(collection_addr, &query_token_msg)
        .unwrap();
    assert_eq!(res.token_uri, token_uri);
}
