use crate::common_setup::{
    contract_boxes::custom_mock_app,
    msg::MinterCollectionResponse,
    setup_accounts_and_block::setup_accounts,
    setup_minter::common::minter_params::minter_params_token,
    setup_minter::vending_minter::setup::{configure_minter, vending_minter_code_ids},
    templates::vending_minter_per_address_limit,
};
use crate::common_setup::{
    setup_accounts_and_block::{coins_for_msg, setup_block_time},
    templates::vending_minter_template,
};
use cosmwasm_std::{coin, coins, Coin, Timestamp, Uint128};
use cw721::{Cw721QueryMsg, OwnerOfResponse, TokensResponse};
use cw_multi_test::Executor;
use sg2::tests::mock_collection_params_1;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
use vending_minter::{
    msg::{ExecuteMsg, MintableNumTokensResponse, QueryMsg},
    ContractError,
};
const MINT_PRICE: u128 = 100_000_000;
const ADMIN_MINT_PRICE: u128 = 0;

#[test]
fn check_per_address_limit() {
    let vt = vending_minter_template(2);
    let (mut router, creator, buyer) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();
    // Set to genesis mint start time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME, None);

    // Set limit, check unauthorized
    let per_address_limit_msg = ExecuteMsg::UpdatePerAddressLimit {
        per_address_limit: 30,
    };
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &per_address_limit_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_err());

    // Set limit errors, invalid limit == 0
    let per_address_limit_msg = ExecuteMsg::UpdatePerAddressLimit {
        per_address_limit: 0,
    };
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &per_address_limit_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_err());

    // Set limit errors, invalid limit over max
    let per_address_limit_msg = ExecuteMsg::UpdatePerAddressLimit {
        per_address_limit: 100,
    };
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &per_address_limit_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_err());

    // Set limit succeeds, mint fails, over max
    let per_address_limit_msg = ExecuteMsg::UpdatePerAddressLimit {
        per_address_limit: 1,
    };
    let res = router.execute_contract(creator, minter_addr.clone(), &per_address_limit_msg, &[]);
    assert!(res.is_ok());

    // First mint succeeds
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );

    assert!(res.is_ok());

    // Second mint fails from exceeding per address limit
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer,
        minter_addr,
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_err());
}

#[test]
fn check_per_address_limit_3_percent() {
    let vt = vending_minter_template(200);
    let (mut router, creator, buyer) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();
    // Set to genesis mint start time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME, None);

    // 3% of 200 is 6, cant have 7 per address limit
    let per_address_limit_msg = ExecuteMsg::UpdatePerAddressLimit {
        per_address_limit: 7,
    };
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &per_address_limit_msg,
        &[],
    );
    assert!(res.is_err());

    // can set to 3% which is 6
    let per_address_limit_msg = ExecuteMsg::UpdatePerAddressLimit {
        per_address_limit: 6,
    };
    let res = router.execute_contract(creator, minter_addr.clone(), &per_address_limit_msg, &[]);
    assert!(res.is_ok());

    for _ in 0..6 {
        let mint_msg = ExecuteMsg::Mint {};
        let res = router.execute_contract(
            buyer.clone(),
            minter_addr.clone(),
            &mint_msg,
            &coins(MINT_PRICE, NATIVE_DENOM),
        );
        assert!(res.is_ok());
    }

    // Second mint fails from exceeding per address limit
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer,
        minter_addr,
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_err());
}

#[test]
fn check_allow_3_mint_when_less_than_100_size_collection() {
    let vt = vending_minter_template(25);
    let (mut router, creator, _) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();
    // Set to genesis mint start time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME, None);

    // can't set to 4 according to per_address_limit <= 3 check
    let per_address_limit_msg = ExecuteMsg::UpdatePerAddressLimit {
        per_address_limit: 4,
    };
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &per_address_limit_msg,
        &[],
    );
    assert!(res.is_err());

    // can set to 3 which is above 3% but passes per_address_limit <= 3  check
    let per_address_limit_msg = ExecuteMsg::UpdatePerAddressLimit {
        per_address_limit: 3,
    };
    let res = router.execute_contract(creator, minter_addr, &per_address_limit_msg, &[]);
    assert!(res.is_ok());
}

#[test]
fn check_3_percent_round_up() {
    let vt = vending_minter_template(215);
    let (mut router, creator, buyer) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();
    // Set to genesis mint start time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME, None);

    // 3% of 215 is 6.45, we should round up to 7 allowed, 8 too many
    let per_address_limit_msg = ExecuteMsg::UpdatePerAddressLimit {
        per_address_limit: 8,
    };
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &per_address_limit_msg,
        &[],
    );
    assert!(res.is_err());

    // can set to 3% which is 7
    let per_address_limit_msg = ExecuteMsg::UpdatePerAddressLimit {
        per_address_limit: 7,
    };
    let res = router.execute_contract(creator, minter_addr.clone(), &per_address_limit_msg, &[]);
    assert!(res.is_ok());

    for _ in 0..7 {
        let mint_msg = ExecuteMsg::Mint {};
        let res = router.execute_contract(
            buyer.clone(),
            minter_addr.clone(),
            &mint_msg,
            &coins(MINT_PRICE, NATIVE_DENOM),
        );
        assert!(res.is_ok());
    }

    // Second mint fails from exceeding per address limit
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer,
        minter_addr,
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_err());
}

#[test]
fn check_per_address_limit_above_max_per_address_limit() {
    let vt = vending_minter_template(5000);
    let (mut router, creator, _) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();
    // Set to genesis mint start time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME, None);

    // 3% of 5000 is 150, but we will get blocked by the max per address limit flag
    let per_address_limit_msg = ExecuteMsg::UpdatePerAddressLimit {
        per_address_limit: 51,
    };
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &per_address_limit_msg,
        &[],
    );
    assert!(res.is_err());

    // can set to 50
    let per_address_limit_msg = ExecuteMsg::UpdatePerAddressLimit {
        per_address_limit: 50,
    };
    let res = router.execute_contract(creator, minter_addr, &per_address_limit_msg, &[]);
    assert!(res.is_ok());
}

#[test]
fn check_invalid_update_per_address_limit() {
    let max_per_address_limit = 50;
    let mut router = custom_mock_app();
    let (creator, _) = setup_accounts(&mut router);
    let num_tokens = 300;
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
    let err = minter_collection_response[0].error.as_ref();
    assert!(err.is_none());

    let minter_addr = minter_collection_response[0].minter.clone().unwrap();
    let update_msg = ExecuteMsg::UpdatePerAddressLimit {
        per_address_limit: 110,
    };
    let err = router
        .execute_contract(creator, minter_addr, &update_msg, &[])
        .unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        ContractError::InvalidPerAddressLimit {
            max: max_per_address_limit,
            min: 1,
            got: 110,
        }
        .to_string()
    );
}

#[test]
fn check_invalid_instantiate_per_address_limit() {
    let num_tokens = 400;
    let per_address_limit = 30;
    let vt = vending_minter_per_address_limit(num_tokens, per_address_limit);
    let (mut router, _) = (vt.router, vt.accts.creator);
    let err = vt.collection_response_vec[0].error.as_ref();

    setup_block_time(&mut router, GENESIS_MINT_START_TIME - 1, None);
    assert_eq!(
        err.unwrap().source().unwrap().source().unwrap().to_string(),
        ContractError::InvalidPerAddressLimit {
            max: 12,
            min: 1,
            got: 30
        }
        .to_string()
    );
}

#[test]
fn mint_for_token_id_addr() {
    let vt = vending_minter_template(4);
    let (mut router, creator, buyer) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();
    let collection_addr = vt.collection_response_vec[0].collection.clone().unwrap();
    setup_block_time(&mut router, GENESIS_MINT_START_TIME, None);
    // Try mint_for, test unauthorized
    let mint_for_msg = ExecuteMsg::MintFor {
        token_id: 1,
        recipient: buyer.to_string(),
    };
    let err = router
        .execute_contract(
            buyer.clone(),
            minter_addr.clone(),
            &mint_for_msg,
            &coins_for_msg(Coin {
                amount: Uint128::from(ADMIN_MINT_PRICE),
                denom: NATIVE_DENOM.to_string(),
            }),
        )
        .unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        ContractError::Unauthorized("Sender is not an admin".to_string()).to_string(),
    );

    // Test token id already sold
    // 1. random mint token_id
    // 2. mint_for same token_id
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_ok());

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
    let sold_token_id: u32 = res.tokens[0].parse::<u32>().unwrap();

    // Minter contract should have a balance
    let minter_balance = router
        .wrap()
        .query_all_balances(minter_addr.clone())
        .unwrap();
    assert_eq!(0, minter_balance.len());

    // Mint fails, invalid token_id
    let token_id: u32 = 0;
    let mint_for_msg = ExecuteMsg::MintFor {
        token_id,
        recipient: buyer.to_string(),
    };
    let err = router
        .execute_contract(
            creator.clone(),
            minter_addr.clone(),
            &mint_for_msg,
            &coins_for_msg(Coin {
                amount: Uint128::from(ADMIN_MINT_PRICE),
                denom: NATIVE_DENOM.to_string(),
            }),
        )
        .unwrap_err();
    assert_eq!(
        ContractError::InvalidTokenId {}.to_string(),
        err.source().unwrap().to_string()
    );

    // Mint fails, token_id already sold
    let mint_for_msg = ExecuteMsg::MintFor {
        token_id: sold_token_id,
        recipient: buyer.to_string(),
    };
    let err = router
        .execute_contract(
            creator.clone(),
            minter_addr.clone(),
            &mint_for_msg,
            &coins_for_msg(Coin {
                amount: Uint128::from(ADMIN_MINT_PRICE),
                denom: NATIVE_DENOM.to_string(),
            }),
        )
        .unwrap_err();
    assert_eq!(
        ContractError::TokenIdAlreadySold {
            token_id: sold_token_id
        }
        .to_string(),
        err.source().unwrap().to_string()
    );

    let mintable_num_tokens_response: MintableNumTokensResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &QueryMsg::MintableNumTokens {})
        .unwrap();
    assert_eq!(mintable_num_tokens_response.count, 3);

    // Mint fails, wrong admin airdrop price
    let err = router
        .execute_contract(
            creator.clone(),
            minter_addr.clone(),
            &mint_for_msg,
            &coins_for_msg(Coin {
                amount: Uint128::from(ADMIN_MINT_PRICE + 1),
                denom: NATIVE_DENOM.to_string(),
            }),
        )
        .unwrap_err();
    assert_eq!(
        ContractError::IncorrectPaymentAmount(
            coin(ADMIN_MINT_PRICE + 1, NATIVE_DENOM.to_string()),
            coin(ADMIN_MINT_PRICE, NATIVE_DENOM.to_string())
        )
        .to_string(),
        err.source().unwrap().to_string()
    );

    // Test mint_for token_id 2 then normal mint
    let token_id = 2;
    let mint_for_msg = ExecuteMsg::MintFor {
        token_id,
        recipient: buyer.to_string(),
    };
    let res = router.execute_contract(
        creator,
        minter_addr.clone(),
        &mint_for_msg,
        &coins_for_msg(Coin {
            amount: Uint128::from(ADMIN_MINT_PRICE),
            denom: NATIVE_DENOM.to_string(),
        }),
    );
    assert!(res.is_ok());

    let res: OwnerOfResponse = router
        .wrap()
        .query_wasm_smart(
            collection_addr,
            &Cw721QueryMsg::OwnerOf {
                token_id: 2.to_string(),
                include_expired: None,
            },
        )
        .unwrap();
    assert_eq!(res.owner, buyer.to_string());

    let mintable_num_tokens_response: MintableNumTokensResponse = router
        .wrap()
        .query_wasm_smart(minter_addr, &QueryMsg::MintableNumTokens {})
        .unwrap();
    assert_eq!(mintable_num_tokens_response.count, 2);
}
