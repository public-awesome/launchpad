use cosmwasm_std::coins;
use cw721::msg::{NftInfoResponse, TokensResponse};
use cw721::EmptyOptionalNftExtension;
use cw721_base::msg::ExecuteMsg as Cw721ExecuteMsg;
use cw_multi_test::Executor;
use sg_utils::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
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
    let query_tokens_msg = cw721_base::msg::QueryMsg::Tokens {
        owner: buyer.to_string(),
        start_after: None,
        limit: None,
    };
    let res: TokensResponse = router
        .wrap()
        .query_wasm_smart(collection_addr.clone(), &query_tokens_msg)
        .unwrap();
    let token_id = res.tokens[0].to_string();

    // update token metadata using cw721 UpdateNftInfo
    let token_uri = Some("ipfs://new_token_uri".to_string());
    let msg = Cw721ExecuteMsg::UpdateNftInfo {
        token_id: token_id.clone(),
        token_uri: token_uri.clone(),
        extension: None,
    };
    let res = router.execute_contract(creator, collection_addr.clone(), &msg, &[]);
    assert!(res.is_ok());

    // check token metadata
    let res: TokensResponse = router
        .wrap()
        .query_wasm_smart(collection_addr.clone(), &query_tokens_msg)
        .unwrap();
    assert_eq!(res.tokens[0], token_id);
    let query_token_msg = cw721_base::msg::QueryMsg::NftInfo { token_id };
    let res: NftInfoResponse<EmptyOptionalNftExtension> = router
        .wrap()
        .query_wasm_smart(collection_addr, &query_token_msg)
        .unwrap();
    assert_eq!(res.token_uri, token_uri);
}
