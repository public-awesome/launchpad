use cosmwasm_std::Addr;
use cosmwasm_std::{coin, to_binary, Coin, Uint128};
use cw721::{Cw721ExecuteMsg, Cw721QueryMsg};
use cw_multi_test::Executor;
use open_edition_factory::state::ParamsExtension;
use open_edition_minter::msg::ExecuteMsg;
use sg_std::GENESIS_MINT_START_TIME;
use sg_std::NATIVE_DENOM;

use crate::common_setup::setup_accounts_and_block::setup_block_time;
use crate::common_setup::{
    setup_minter::{
        common::constants::DEV_ADDRESS,
        open_edition_minter::minter_params::{default_nft_data, init_msg},
    },
    templates::open_edition_minter_with_two_sg721_collections_burn_mint,
};

#[test]
fn check_burns_tokens_when_received() {
    let params_extension = ParamsExtension {
        max_per_address_limit: 10,
        airdrop_mint_fee_bps: 100,
        airdrop_mint_price: Coin {
            denom: NATIVE_DENOM.to_string(),
            amount: Uint128::new(100_000_000u128),
        },
        dev_fee_address: DEV_ADDRESS.to_string(),
    };
    let per_address_limit_minter = Some(2);
    let init_msg = init_msg(
        default_nft_data(),
        per_address_limit_minter,
        None,
        None,
        None,
    );
    let allowed_collections = vec![Addr::unchecked("contract2")];
    let vt = open_edition_minter_with_two_sg721_collections_burn_mint(
        params_extension,
        init_msg,
        allowed_collections,
    )
    .unwrap();
    let (mut router, creator) = (vt.router, vt.accts.creator);
    let minter_addr_1 = vt.collection_response_vec[0].minter.clone().unwrap();
    let collection_addr_1 = vt.collection_response_vec[0].collection.clone().unwrap();

    let minter_addr_2 = vt.collection_response_vec[1].minter.clone().unwrap();

    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 101, None);

    let mint_price = 100_000_000;
    // Mint one NFT
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        creator.clone(),
        minter_addr_1,
        &mint_msg,
        &[coin(mint_price, NATIVE_DENOM)],
    );
    assert!(res.is_ok());

    let num_tokens_res: cw721::NumTokensResponse = router
        .wrap()
        .query_wasm_smart(collection_addr_1.clone(), &Cw721QueryMsg::NumTokens {})
        .unwrap();
    // one token after mint
    assert_eq!(num_tokens_res.count, 1);

    let send_nft = Cw721ExecuteMsg::SendNft {
        contract: minter_addr_2.to_string(),
        token_id: 1.to_string(),
        msg: to_binary("this is a test").unwrap(),
    };

    let res = router.execute_contract(creator, collection_addr_1.clone(), &send_nft, &[]);
    println!("res is {:?}", res);
    assert!(res.is_ok());

    let num_tokens_res: cw721::NumTokensResponse = router
        .wrap()
        .query_wasm_smart(collection_addr_1, &Cw721QueryMsg::NumTokens {})
        .unwrap();

    // zero tokens after burn
    assert_eq!(num_tokens_res.count, 0);
}

#[test]
fn check_mints_new_tokens_when_received() {
    let params_extension = ParamsExtension {
        max_per_address_limit: 10,
        airdrop_mint_fee_bps: 100,
        airdrop_mint_price: Coin {
            denom: NATIVE_DENOM.to_string(),
            amount: Uint128::new(100_000_000u128),
        },
        dev_fee_address: DEV_ADDRESS.to_string(),
    };
    let per_address_limit_minter = Some(2);
    let init_msg = init_msg(
        default_nft_data(),
        per_address_limit_minter,
        None,
        None,
        None,
    );
    let allowed_collections = vec![Addr::unchecked("contract2")];
    let vt = open_edition_minter_with_two_sg721_collections_burn_mint(
        params_extension,
        init_msg,
        allowed_collections,
    )
    .unwrap();
    let (mut router, creator) = (vt.router, vt.accts.creator);
    let minter_addr_1 = vt.collection_response_vec[0].minter.clone().unwrap();
    let collection_addr_1 = vt.collection_response_vec[0].collection.clone().unwrap();

    let minter_addr_2 = vt.collection_response_vec[1].minter.clone().unwrap();
    let collection_addr_2 = vt.collection_response_vec[1].collection.clone().unwrap();

    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 101, None);
    // Mint one NFT
    let mint_price = 100_000_000;

    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        creator.clone(),
        minter_addr_1,
        &mint_msg,
        &[coin(mint_price, NATIVE_DENOM)],
    );
    assert!(res.is_ok());

    let num_tokens_res: cw721::NumTokensResponse = router
        .wrap()
        .query_wasm_smart(collection_addr_1.clone(), &Cw721QueryMsg::NumTokens {})
        .unwrap();
    // one token after mint
    assert_eq!(num_tokens_res.count, 1);

    let res = router.execute_contract(
        creator.clone(),
        minter_addr_2.clone(),
        &mint_msg,
        &[coin(mint_price, NATIVE_DENOM)],
    );
    assert!(res.is_ok());

    let num_tokens_res: cw721::NumTokensResponse = router
        .wrap()
        .query_wasm_smart(collection_addr_2.clone(), &Cw721QueryMsg::NumTokens {})
        .unwrap();
    // one token after mint
    assert_eq!(num_tokens_res.count, 1);

    let send_nft = Cw721ExecuteMsg::SendNft {
        contract: minter_addr_2.to_string(),
        token_id: 1.to_string(),
        msg: to_binary("this is a test").unwrap(),
    };
    let res = router.execute_contract(
        creator,
        collection_addr_1.clone(),
        &send_nft,
        &[coin(mint_price, NATIVE_DENOM)],
    );
    assert!(res.is_ok());
    let num_tokens_res: cw721::NumTokensResponse = router
        .wrap()
        .query_wasm_smart(collection_addr_1, &Cw721QueryMsg::NumTokens {})
        .unwrap();
    // one token after mint
    assert_eq!(num_tokens_res.count, 0);

    let num_tokens_res: cw721::NumTokensResponse = router
        .wrap()
        .query_wasm_smart(collection_addr_2, &Cw721QueryMsg::NumTokens {})
        .unwrap();
    // one token after mint
    assert_eq!(num_tokens_res.count, 2);
}
