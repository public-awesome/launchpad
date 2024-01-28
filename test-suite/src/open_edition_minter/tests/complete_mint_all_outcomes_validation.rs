use cosmwasm_std::{coins, Coin, Timestamp, Uint128};
use cw721::{Cw721QueryMsg, NumTokensResponse, OwnerOfResponse};
use cw_multi_test::{BankSudo, Executor, SudoMsg};
use open_edition_factory::state::ParamsExtension;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};

use open_edition_minter::msg::{
    ConfigResponse, EndTimeResponse, ExecuteMsg, MintCountResponse, MintPriceResponse,
    MintableNumTokensResponse, QueryMsg, StartTimeResponse, TotalMintCountResponse,
};
use sg4::StatusResponse;

use crate::common_setup::setup_accounts_and_block::{coins_for_msg, setup_block_time};
use crate::common_setup::setup_minter::common::constants::DEV_ADDRESS;
use crate::common_setup::setup_minter::open_edition_minter::minter_params::{
    default_nft_data, init_msg,
};
use crate::common_setup::templates::open_edition_minter_custom_template;

const MINT_PRICE: u128 = 100_000_000;

fn check_mint_revenues_distribution(num_tokens: Option<u32>, end_minter_time: Option<Timestamp>) {
    let params_extension = ParamsExtension {
        max_token_limit: 10,
        max_per_address_limit: 10,
        airdrop_mint_fee_bps: 100,
        airdrop_mint_price: Coin {
            denom: NATIVE_DENOM.to_string(),
            amount: Uint128::new(100_000_000u128),
        },
        dev_fee_address: DEV_ADDRESS.to_string(),
    };
    let per_address_limit_minter = Some(3);
    let init_msg = init_msg(
        default_nft_data(),
        per_address_limit_minter,
        None,
        end_minter_time,
        num_tokens,
        None,
    );
    let vt = open_edition_minter_custom_template(params_extension, init_msg).unwrap();

    let (mut router, creator, buyer) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();
    let collection_addr = vt.collection_response_vec[0].collection.clone().unwrap();

    // Set to genesis mint start time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 101, None);

    let initial_buyer_balances = router.wrap().query_all_balances(buyer.clone()).unwrap();
    let initial_creator_balances = router.wrap().query_all_balances(creator.clone()).unwrap();
    assert_eq!(
        initial_creator_balances[0].amount,
        Uint128::new(2_000_000_000)
    );
    let initial_dev_balances = router.wrap().query_all_balances(DEV_ADDRESS).unwrap();
    assert_eq!(initial_dev_balances[0].amount, Uint128::new(2_000_000_000));

    // Query Start Time
    // We know it is GENESIS_MINT_START_TIME + 100
    let query_start_time_msg: QueryMsg = QueryMsg::StartTime {};
    let res: StartTimeResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &query_start_time_msg)
        .unwrap();
    assert_eq!(
        res.start_time,
        Timestamp::from_nanos(GENESIS_MINT_START_TIME + 100).to_string()
    );

    // Query End Time
    // We know it is GENESIS_MINT_START_TIME + 10_000
    let query_end_time_msg: QueryMsg = QueryMsg::EndTime {};
    let res: EndTimeResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &query_end_time_msg)
        .unwrap();
    if end_minter_time.is_some() {
        assert_eq!(
            res.end_time,
            Some(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10_000).to_string())
        );
    } else {
        assert_eq!(res.end_time, None);
    }

    // Query the Max Tokens or End Time depending on which test is executed
    let query_config_msg = QueryMsg::MintableNumTokens {};
    let res: MintableNumTokensResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &query_config_msg)
        .unwrap();
    if end_minter_time.is_some() {
        assert_eq!(res.count, None);
    } else {
        assert_eq!(res.count, Some(5));
    }

    // Query the Config info
    let query_config_msg = QueryMsg::Config {};
    let res: ConfigResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &query_config_msg)
        .unwrap();
    if end_minter_time.is_some() {
        assert_eq!(res.num_tokens, None);
        assert_eq!(res.end_time, end_minter_time);
    } else {
        assert_eq!(res.num_tokens, Some(5));
        assert_eq!(res.end_time, None);
    }

    // Query Total Minted Tokens -> Should be 0 at the start
    let query_total_minted_msg: QueryMsg = QueryMsg::TotalMintCount {};
    let res: TotalMintCountResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &query_total_minted_msg)
        .unwrap();
    assert_eq!(res.count, 0u32);

    // Invalid price
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(100u128, NATIVE_DENOM),
    );
    assert_eq!(
        res.err().unwrap().source().unwrap().to_string(),
        "IncorrectPaymentAmount 100ustars != 100000000ustars"
    );

    // Invalid price
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(200_000_000u128, NATIVE_DENOM),
    );
    assert_eq!(
        res.err().unwrap().source().unwrap().to_string(),
        "IncorrectPaymentAmount 200000000ustars != 100000000ustars"
    );

    // Invalid price
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(buyer.clone(), minter_addr.clone(), &mint_msg, &[]);
    assert_eq!(
        res.err().unwrap().source().unwrap().to_string(),
        "IncorrectPaymentAmount 0ustars != 100000000ustars"
    );

    // Invalid denom
    router
        .sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: buyer.to_string(),
                amount: coins(100_000u128, "invalid".to_string()),
            }
        }))
        .map_err(|err| println!("{err:?}"))
        .ok();
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(100u128, "invalid".to_string()),
    );
    assert_eq!(
        res.err().unwrap().source().unwrap().to_string(),
        "Received unsupported denom 'invalid'"
    );

    for _i in 1..=2 {
        let mint_msg = ExecuteMsg::Mint {};
        let res = router.execute_contract(
            buyer.clone(),
            minter_addr.clone(),
            &mint_msg,
            &coins(MINT_PRICE, NATIVE_DENOM),
        );
        assert!(res.is_ok());
    }

    // Buyer should be -100 x2 stars
    let buyer_balances = router.wrap().query_all_balances(buyer.clone()).unwrap();
    assert_eq!(
        buyer_balances[1].amount,
        initial_buyer_balances[0].amount - Uint128::new(200_000_000u128)
    );

    // Creator should be at +100 x2 stars - mint fees (currently at 10 x2) [Mint fees include Dev fees]
    let creator_balances = router.wrap().query_all_balances(creator.clone()).unwrap();
    assert_eq!(
        creator_balances[0].amount,
        initial_creator_balances[0].amount + Uint128::new(200_000_000 - 20_000_000)
    );

    // Mint fees / mint = 10_000_000 where 50% is toward the fair burn pool and 50% is
    // toward the dev so the dev should get 10_000_000 * 0.5 = 5_000_000 / mint
    let dev_balances = router.wrap().query_all_balances(DEV_ADDRESS).unwrap();
    assert_eq!(
        dev_balances[0].amount,
        initial_dev_balances[0].amount + Uint128::new(5_000_000 * 2)
    );

    // Should be owner of the token -> 2
    let query_owner_msg = Cw721QueryMsg::OwnerOf {
        token_id: String::from("2"),
        include_expired: None,
    };

    let res: OwnerOfResponse = router
        .wrap()
        .query_wasm_smart(collection_addr.clone(), &query_owner_msg)
        .unwrap();
    assert_eq!(res.owner, buyer.to_string());

    // Check mint count
    let num_tokens_msg = Cw721QueryMsg::NumTokens {};
    let res: NumTokensResponse = router
        .wrap()
        .query_wasm_smart(collection_addr, &num_tokens_msg)
        .unwrap();
    assert_eq!(res.count, 2);

    // Check the Start time
    let query_config_msg = QueryMsg::StartTime {};
    let res: StartTimeResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &query_config_msg)
        .unwrap();
    assert_eq!(res.start_time, "1647032400.000000100");

    // Creator mints an extra NFT for the buyer (who is a friend)
    let mint_to_msg = ExecuteMsg::MintTo {
        recipient: buyer.to_string(),
    };
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &mint_to_msg,
        &[Coin {
            amount: Uint128::from(100_000_000u128),
            denom: NATIVE_DENOM.to_string(),
        }],
    );
    assert!(res.is_ok());

    // Mint count is not increased if admin mints for the user -> already had 2 minted
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

    // Check the status
    let query_config_msg = QueryMsg::Status {};
    let res: StatusResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &query_config_msg)
        .unwrap();
    assert!(!res.status.is_blocked);

    // Check the price
    let query_config_msg = QueryMsg::MintPrice {};
    let res: MintPriceResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &query_config_msg)
        .unwrap();
    assert_eq!(
        res.airdrop_price,
        Coin {
            denom: NATIVE_DENOM.to_string(),
            amount: Uint128::new(100_000_000)
        }
    );
    assert_eq!(
        res.current_price,
        Coin {
            denom: NATIVE_DENOM.to_string(),
            amount: Uint128::new(100_000_000)
        }
    );
    assert_eq!(
        res.public_price,
        Coin {
            denom: NATIVE_DENOM.to_string(),
            amount: Uint128::new(100_000_000)
        }
    );

    // Query Total Minted Tokens -> Should be 3 (2 normal mints + MintTo)
    let query_total_minted_msg: QueryMsg = QueryMsg::TotalMintCount {};
    let res: TotalMintCountResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &query_total_minted_msg)
        .unwrap();
    assert_eq!(res.count, 3u32);

    // If time end has been reached, can't mint anymore
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 1_000_000, None);
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    if end_minter_time.is_some() {
        assert_eq!(
            res.err().unwrap().source().unwrap().to_string(),
            "Minting has ended"
        );
    } else {
        assert!(res.is_ok());
    }

    // Try to execute admin only entry point from buyer
    let exec_msg = ExecuteMsg::MintTo {
        recipient: buyer.to_string(),
    };
    let res = router.execute_contract(buyer.clone(), minter_addr.clone(), &exec_msg, &[]);
    assert_eq!(
        res.err().unwrap().source().unwrap().to_string(),
        "Unauthorized: Sender is not an admin"
    );
    let exec_msg = ExecuteMsg::UpdatePerAddressLimit {
        per_address_limit: 200,
    };
    let res = router.execute_contract(buyer.clone(), minter_addr.clone(), &exec_msg, &[]);
    assert_eq!(
        res.err().unwrap().source().unwrap().to_string(),
        "Unauthorized: Sender is not an admin"
    );
    let exec_msg = ExecuteMsg::UpdateStartTradingTime(Some(Timestamp::from_nanos(1)));
    let res = router.execute_contract(buyer.clone(), minter_addr.clone(), &exec_msg, &[]);
    assert_eq!(
        res.err().unwrap().source().unwrap().to_string(),
        "Unauthorized: Sender is not an admin"
    );

    // Minter contract should have no balance
    let minter_balance = router
        .wrap()
        .query_all_balances(minter_addr.clone())
        .unwrap();
    assert_eq!(minter_balance.len(), 0);

    // Creator can't use MintTo if the end time is < block time
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &mint_to_msg,
        &coins_for_msg(Coin {
            amount: Uint128::from(100_000_000u128),
            denom: NATIVE_DENOM.to_string(),
        }),
    );
    if end_minter_time.is_some() {
        assert!(res.is_err());
    } else {
        assert!(res.is_ok());
    }

    // Check if the count is accurate depending on the config of the test
    // if no end time -> should be at 5 otherwise 3 as it would not be possible do use the mint to
    let query_config_msg = QueryMsg::TotalMintCount {};
    let res: TotalMintCountResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &query_config_msg)
        .unwrap();
    if end_minter_time.is_some() {
        assert_eq!(res.count, 3);
    } else {
        assert_eq!(res.count, 5);
    }

    // It should not be possible to mint anymore in both cases
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    if end_minter_time.is_some() {
        assert_eq!(
            res.err().unwrap().source().unwrap().to_string(),
            "Minting has ended"
        );
    } else {
        assert_eq!(res.err().unwrap().source().unwrap().to_string(), "Sold out");
    }

    // Can purge after sold out
    let purge_msg = ExecuteMsg::Purge {};
    let res = router.execute_contract(creator, minter_addr.clone(), &purge_msg, &[]);
    assert!(res.is_ok());

    // MintCount should be 0 after purge
    let res: MintCountResponse = router
        .wrap()
        .query_wasm_smart(
            minter_addr,
            &QueryMsg::MintCount {
                address: buyer.to_string(),
            },
        )
        .unwrap();
    assert_eq!(res.count, 0);
}

#[test]
fn check_mint_revenues_distribution_without_end_time() {
    check_mint_revenues_distribution(
        None,
        Some(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10_000)),
    )
}

#[test]
fn check_mint_revenues_distribution_with_end_time() {
    check_mint_revenues_distribution(Some(5u32), None)
}
