use cosmwasm_std::{Coin, Timestamp, Uint128};
use cw_multi_test::Executor;
use open_edition_factory::state::ParamsExtension;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};

use open_edition_minter::msg::{ExecuteMsg, QueryMsg};

use crate::common_setup::setup_accounts_and_block::setup_block_time;
use crate::common_setup::setup_minter::common::constants::DEV_ADDRESS;
use crate::common_setup::setup_minter::open_edition_minter::minter_params::{
    default_nft_data, init_msg,
};
use crate::common_setup::templates::open_edition_minter_custom_template;

const MINT_PRICE: u128 = 100_000_000;

#[test]
fn check_mint_price_updates() {
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
    let per_address_limit_minter = Some(2);
    let init_msg = init_msg(
        default_nft_data(),
        per_address_limit_minter,
        None,
        Some(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10_000)),
        None,
        None,
    );

    let vt = open_edition_minter_custom_template(params_extension, init_msg).unwrap();
    let (mut router, creator, _buyer) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();

    // Query Mint Price
    let query_mint_price_msg: QueryMsg = QueryMsg::MintPrice {};
    let res: open_edition_minter::msg::MintPriceResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &query_mint_price_msg)
        .unwrap();
    assert_eq!(
        res.current_price,
        Coin {
            denom: NATIVE_DENOM.to_string(),
            amount: Uint128::new(MINT_PRICE)
        }
    );

    assert_eq!(
        res.airdrop_price,
        Coin {
            denom: NATIVE_DENOM.to_string(),
            amount: Uint128::new(MINT_PRICE)
        }
    );
    assert_eq!(
        res.public_price,
        Coin {
            denom: NATIVE_DENOM.to_string(),
            amount: Uint128::new(MINT_PRICE)
        }
    );

    // Change to invalid price
    let update_msg = ExecuteMsg::UpdateMintPrice { price: 1u128 };
    let res = router.execute_contract(creator.clone(), minter_addr.clone(), &update_msg, &[]);
    assert_eq!(
        res.err().unwrap().source().unwrap().to_string(),
        "Minimum network mint price 100000000 got 1"
    );

    // Can increase the price because we are before the start time
    let update_msg = ExecuteMsg::UpdateMintPrice {
        price: MINT_PRICE + 100u128,
    };
    let res = router.execute_contract(creator.clone(), minter_addr.clone(), &update_msg, &[]);
    assert!(res.is_ok());

    // Query the new Mint Price
    let query_mint_price_msg: QueryMsg = QueryMsg::MintPrice {};
    let res: open_edition_minter::msg::MintPriceResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &query_mint_price_msg)
        .unwrap();
    assert_eq!(
        res.current_price,
        Coin {
            denom: NATIVE_DENOM.to_string(),
            amount: Uint128::new(MINT_PRICE + 100u128)
        }
    );
    assert_eq!(
        res.airdrop_price,
        Coin {
            denom: NATIVE_DENOM.to_string(),
            amount: Uint128::new(MINT_PRICE)
        }
    );
    assert_eq!(
        res.public_price,
        Coin {
            denom: NATIVE_DENOM.to_string(),
            amount: Uint128::new(MINT_PRICE + 100u128)
        }
    );

    // Set block forward, after start time. mint succeeds
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 200, None);

    // Should not be able to increase price
    let update_msg = ExecuteMsg::UpdateMintPrice {
        price: MINT_PRICE + 200u128,
    };
    let res = router.execute_contract(creator.clone(), minter_addr.clone(), &update_msg, &[]);
    assert_eq!(
        res.err().unwrap().source().unwrap().to_string(),
        "Update price 100000200 higher than allowed price 100000100"
    );

    // Decrease the price after the start time
    let update_msg = ExecuteMsg::UpdateMintPrice { price: MINT_PRICE };
    let res = router.execute_contract(creator.clone(), minter_addr.clone(), &update_msg, &[]);
    assert!(res.is_ok());

    // Query the new Mint Price
    let query_mint_price_msg: QueryMsg = QueryMsg::MintPrice {};
    let res: open_edition_minter::msg::MintPriceResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &query_mint_price_msg)
        .unwrap();
    assert_eq!(
        res.current_price,
        Coin {
            denom: NATIVE_DENOM.to_string(),
            amount: Uint128::new(MINT_PRICE)
        }
    );
    assert_eq!(
        res.airdrop_price,
        Coin {
            denom: NATIVE_DENOM.to_string(),
            amount: Uint128::new(MINT_PRICE)
        }
    );
    assert_eq!(
        res.public_price,
        Coin {
            denom: NATIVE_DENOM.to_string(),
            amount: Uint128::new(MINT_PRICE)
        }
    );

    // If we are past the end time - cannot mint and cant change price either
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 50_000, None);

    // Try to change the price
    let update_msg = ExecuteMsg::UpdateMintPrice { price: MINT_PRICE };
    let res = router.execute_contract(creator, minter_addr, &update_msg, &[]);
    assert_eq!(
        res.err().unwrap().source().unwrap().to_string(),
        "Minting has ended"
    );
}
