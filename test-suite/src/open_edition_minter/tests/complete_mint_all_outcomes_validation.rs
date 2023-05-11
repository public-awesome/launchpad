use cosmwasm_std::{Coin, coins, Timestamp, Uint128};
use cw721::{Cw721QueryMsg, OwnerOfResponse};
use cw_multi_test::{BankSudo, Executor, SudoMsg};
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};

use open_edition_minter::{
    msg::{ExecuteMsg, QueryMsg},
};
use open_edition_minter::msg::{ConfigResponse, MintCountResponse, MintPriceResponse, StartTimeResponse};
use sg4::StatusResponse;

use crate::common_setup::{
    setup_accounts_and_block::{coins_for_msg, setup_block_time},
};
use crate::common_setup::setup_minter::common::constants::DEV_ADDRESS;
use crate::common_setup::templates::open_edition_minter_custom_template;

const MINT_PRICE: u128 = 100_000_000;

#[test]
fn check_mint_revenues_distribution() {

    let vt = open_edition_minter_custom_template(
        None,
        None,
        None,
        None,
        Some(3),
        None,
        None,
        None
    ).unwrap();
    let (mut router, creator, buyer) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();
    let collection_addr = vt.collection_response_vec[0].collection.clone().unwrap();

    // Set to genesis mint start time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 101, None);

    let initial_buyer_balances = router.wrap().query_all_balances(buyer.clone()).unwrap();
    let initial_creator_balances = router.wrap().query_all_balances(creator.clone()).unwrap();
    assert_eq!(initial_creator_balances[0].amount, Uint128::new(2_000_000_000));
    let initial_dev_balances = router.wrap().query_all_balances(DEV_ADDRESS.clone()).unwrap();
    assert_eq!(initial_dev_balances[0].amount, Uint128::new(2_000_000_000));

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
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &[],
    );
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
        .map_err(|err| println!("{:?}", err))
        .ok();
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(100u128, "invalid".to_string())
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

    // The fair burn is the fixed burn pct (currently 50%) - fixed dev fee (currently 10% but will be 50%)
    // For 1 mint -> 100_000_000 * 0.1 = 10_000_000 -> 40% is burned and 10% to the dev
    // burn = 4_000_000
    // dev = 1_000_000
    let dev_balances = router.wrap().query_all_balances(DEV_ADDRESS).unwrap();
    assert_eq!(
        dev_balances[0].amount,
        initial_dev_balances[0].amount + Uint128::new(1_000_000 * 2)
    );

    // Should be owner of the token -> 2
    let query_owner_msg = Cw721QueryMsg::OwnerOf {
        token_id: String::from("2"),
        include_expired: None,
    };

    let res: OwnerOfResponse = router
        .wrap()
        .query_wasm_smart(collection_addr, &query_owner_msg)
        .unwrap();
    assert_eq!(res.owner, buyer.to_string());

    // Check the Config
    let query_config_msg = QueryMsg::Config {};
    let res: ConfigResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &query_config_msg)
        .unwrap();
    assert_eq!(res.minted_count, 2);

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
    assert_eq!(res.airdrop_price, Coin { denom: NATIVE_DENOM.to_string(), amount: Uint128::new(100_000_000) });
    assert_eq!(res.current_price, Coin { denom: NATIVE_DENOM.to_string(), amount: Uint128::new(100_000_000) });
    assert_eq!(res.public_price, Coin { denom: NATIVE_DENOM.to_string(), amount: Uint128::new(100_000_000) });

    // If time end has been reached, can't mint anymore
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 1_000_000, None);
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert_eq!(
        res.err().unwrap().source().unwrap().to_string(),
        "Minting has ended"
    );
    
    // Try to execute admin only entry point from buyer
    let exec_msg = ExecuteMsg::MintTo { recipient: buyer.to_string() };
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &exec_msg,
        &[],
    );
    assert_eq!(
        res.err().unwrap().source().unwrap().to_string(),
        "Unauthorized: Sender is not an admin"
    );
    let exec_msg = ExecuteMsg::UpdatePerAddressLimit { per_address_limit: 200 };
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &exec_msg,
        &[],
    );
    assert_eq!(
        res.err().unwrap().source().unwrap().to_string(),
        "Unauthorized: Sender is not an admin"
    );
    let exec_msg = ExecuteMsg::UpdateStartTradingTime(Some(Timestamp::from_nanos(1)));
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &exec_msg,
        &[],
    );
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

    // Creator can't use MintTo if sold out
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &mint_to_msg,
        &coins_for_msg(Coin {
            amount: Uint128::from(100_000_000u128),
            denom: NATIVE_DENOM.to_string(),
        }),
    );
    assert!(res.is_err());

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