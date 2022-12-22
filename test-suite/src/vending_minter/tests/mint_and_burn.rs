use crate::common_setup::templates::{vending_minter_template, vending_minter_with_start_time};
use cosmwasm_std::Coin;
use cosmwasm_std::{coins, Timestamp, Uint128};
use cw_multi_test::Executor;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
use vending_minter::msg::{ExecuteMsg, MintCountResponse, QueryMsg, StartTimeResponse};
use vending_minter::ContractError;

use crate::common_setup::setup_accounts_and_block::{coins_for_msg, setup_block_time};
const INITIAL_BALANCE: u128 = 2_000_000_000;

const MINT_PRICE: u128 = 100_000_000;
const MINT_FEE: u128 = 10_000_000;
const ADMIN_MINT_PRICE: u128 = 0;
#[test]
fn update_mint_price() {
    let vt = vending_minter_template(10);
    let (mut router, creator, buyer) = (vt.router, vt.creator, vt.buyer);
    let minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();
    setup_block_time(&mut router, GENESIS_MINT_START_TIME - 10, None);

    // Update mint price higher
    let update_msg = ExecuteMsg::UpdateMintPrice {
        price: MINT_PRICE + 1,
    };
    let res = router.execute_contract(creator.clone(), minter_addr.clone(), &update_msg, &[]);
    assert!(res.is_ok());

    // Update mint price lower than initial price
    let update_msg = ExecuteMsg::UpdateMintPrice {
        price: MINT_PRICE - 2,
    };
    let res = router.execute_contract(creator.clone(), minter_addr.clone(), &update_msg, &[]);
    assert!(res.is_ok());

    // Update mint price lower than min price
    let update_msg = ExecuteMsg::UpdateMintPrice { price: 1 };
    let res = router.execute_contract(creator.clone(), minter_addr.clone(), &update_msg, &[]);
    assert!(res.is_err());

    // Update mint price higher
    let update_msg = ExecuteMsg::UpdateMintPrice {
        price: MINT_PRICE - 1,
    };
    let res = router.execute_contract(creator.clone(), minter_addr.clone(), &update_msg, &[]);
    assert!(res.is_ok());

    // Set block forward, after start time. mint succeeds
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 10_000_000, None);

    // Update mint price higher after start time, throw error
    let update_msg = ExecuteMsg::UpdateMintPrice { price: MINT_PRICE };
    let err = router
        .execute_contract(creator.clone(), minter_addr.clone(), &update_msg, &[])
        .unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        ContractError::UpdatedMintPriceTooHigh {
            allowed: MINT_PRICE - 1,
            updated: MINT_PRICE
        }
        .to_string()
    );
    // Update mint price lower
    let update_msg = ExecuteMsg::UpdateMintPrice {
        price: MINT_PRICE - 2,
    };
    let res = router.execute_contract(creator, minter_addr.clone(), &update_msg, &[]);
    assert!(res.is_ok());

    // Mint succeeds
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer,
        minter_addr,
        &mint_msg,
        &coins(MINT_PRICE - 2, NATIVE_DENOM),
    );
    assert!(res.is_ok());
}

#[test]
fn burn_remaining() {
    let vt =
        vending_minter_with_start_time(5000, Timestamp::from_nanos(GENESIS_MINT_START_TIME - 1));
    let (mut router, creator, buyer) = (vt.router, vt.creator, vt.buyer);
    let minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();
    // Default start time genesis mint time
    let res: StartTimeResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &QueryMsg::StartTime {})
        .unwrap();
    assert_eq!(
        res.start_time,
        Timestamp::from_nanos(GENESIS_MINT_START_TIME).to_string()
    );

    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 1, None);

    // Succeeds if funds are sent
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_ok());

    // Balances are correct
    // The creator should get the unit price - mint fee for the mint above
    let creator_balances = router.wrap().query_all_balances(creator.clone()).unwrap();
    assert_eq!(
        creator_balances,
        coins(INITIAL_BALANCE + MINT_PRICE - MINT_FEE, NATIVE_DENOM)
    );
    // The buyer's tokens should reduce by unit price
    let buyer_balances = router.wrap().query_all_balances(buyer.clone()).unwrap();
    assert_eq!(
        buyer_balances,
        coins(INITIAL_BALANCE - MINT_PRICE, NATIVE_DENOM)
    );

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

    // Buyer can't call MintTo
    let mint_to_msg = ExecuteMsg::MintTo {
        recipient: buyer.to_string(),
    };
    // Creator mints an extra NFT for the buyer (who is a friend)
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &mint_to_msg,
        &coins_for_msg(Coin {
            amount: Uint128::from(ADMIN_MINT_PRICE),
            denom: NATIVE_DENOM.to_string(),
        }),
    );
    assert!(res.is_ok());

    // Mint count is not increased if admin mints for the user
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

    // Minter contract should have no balance
    let minter_balance = router
        .wrap()
        .query_all_balances(minter_addr.clone())
        .unwrap();
    assert_eq!(0, minter_balance.len());

    let burn_msg = ExecuteMsg::BurnRemaining {};
    // Creator burns remaining supply
    let res = router.execute_contract(creator.clone(), minter_addr.clone(), &burn_msg, &[]);
    assert!(res.is_ok());
    let burn_msg = ExecuteMsg::BurnRemaining {};
    //  Creator burns remaining supply again but should return sold out
    let err = router
        .execute_contract(creator.clone(), minter_addr.clone(), &burn_msg, &[])
        .unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        ContractError::SoldOut {}.to_string()
    );

    // Errors if sold out
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer,
        minter_addr.clone(),
        &mint_msg,
        &coins_for_msg(Coin {
            amount: Uint128::from(MINT_PRICE),
            denom: NATIVE_DENOM.to_string(),
        }),
    );
    assert!(res.is_err());

    // Creator can't use MintTo if sold out
    let res = router.execute_contract(
        creator,
        minter_addr,
        &mint_to_msg,
        &coins_for_msg(Coin {
            amount: Uint128::from(ADMIN_MINT_PRICE),
            denom: NATIVE_DENOM.to_string(),
        }),
    );
    assert!(res.is_err());
}
