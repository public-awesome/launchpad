use cosmwasm_std::{coin, Addr, Coin, Decimal, Timestamp, Uint128};
use cw_multi_test::{BankSudo, Executor, SudoMsg};
use open_edition_factory::state::{OpenEditionMinterParams, ParamsExtension};
use open_edition_minter::msg::ExecuteMsg;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};

use crate::common_setup::{
    setup_accounts_and_block::setup_block_time,
    setup_minter::{
        common::constants::{
            CREATION_FEE, DEV_ADDRESS, FOUNDATION, MINT_FEE_FAIR_BURN, MIN_MINT_PRICE_OPEN_EDITION,
        },
        open_edition_minter::minter_params::{default_nft_data, init_msg},
    },
    templates::open_edition_minter_ibc_template,
};

#[test]
fn check_custom_create_minter_denom() {
    // allow ibc/frenz denom
    let denom = "ibc/frenz";
    let mint_price = coin(MIN_MINT_PRICE_OPEN_EDITION, denom.to_string());
    let params_extension = ParamsExtension {
        max_token_limit: 10,
        max_per_address_limit: 10,
        airdrop_mint_fee_bps: 100,
        airdrop_mint_price: Coin {
            denom: denom.to_string(),
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
        Some(mint_price.clone()),
    );
    let custom_minter_params = OpenEditionMinterParams {
        code_id: 1,
        allowed_sg721_code_ids: vec![1, 3, 5, 6],
        frozen: false,
        creation_fee: coin(CREATION_FEE, NATIVE_DENOM),
        min_mint_price: init_msg.mint_price.clone(),
        mint_fee_bps: MINT_FEE_FAIR_BURN,
        max_trading_offset_secs: 60 * 60 * 24 * 7,
        extension: ParamsExtension {
            max_token_limit: 10,
            max_per_address_limit: 10,
            airdrop_mint_fee_bps: 100,
            dev_fee_address: DEV_ADDRESS.to_string(),
            airdrop_mint_price: params_extension.airdrop_mint_price.clone(),
        },
    };
    let vt =
        open_edition_minter_ibc_template(params_extension, init_msg, custom_minter_params).unwrap();

    let (mut router, creator, buyer) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();
    // give the buyer some of the IBC asset
    router
        .sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: buyer.to_string(),
                amount: vec![mint_price.clone()],
            }
        }))
        .map_err(|err| println!("{err:?}"))
        .ok();

    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 100, None);
    //     // Mint succeeds
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(buyer.clone(), minter_addr, &mint_msg, &[mint_price.clone()]);
    assert!(res.is_ok());

    // confirm balances
    // confirm buyer IBC assets spent
    let balance = router.wrap().query_balance(buyer, denom).unwrap();
    assert_eq!(balance.amount, Uint128::zero());
    // TODO only for noble, seller has 90% IBC asset
    let network_fee = mint_price.amount * Decimal::percent(10);
    let seller_amount = mint_price.amount.checked_sub(network_fee).unwrap();
    let balance = router.wrap().query_balance(creator, denom).unwrap();
    assert_eq!(balance.amount, seller_amount);
    // all mint goes to fairburn_pool confirmed in e2e test
}

#[test]
fn one_hundred_percent_burned_ibc_minter() {
    // factory needs airdrop_mint_price: 0
    // factory needs mint_fee_bps: 100_00 (100%)
    // 100% fairburn, so 50% goes to dev, 50% goes to community pool

    // allow ibc/frenz denom
    let denom = "ibc/frenz";
    let mint_price = coin(MIN_MINT_PRICE_OPEN_EDITION, denom.to_string());
    let params_extension = ParamsExtension {
        max_token_limit: 10,
        max_per_address_limit: 10,
        airdrop_mint_fee_bps: 100,
        airdrop_mint_price: Coin {
            denom: denom.to_string(),
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
        Some(mint_price.clone()),
    );

    let custom_minter_params = OpenEditionMinterParams {
        code_id: 1,
        allowed_sg721_code_ids: vec![1, 3, 5, 6],
        frozen: false,
        creation_fee: coin(CREATION_FEE, NATIVE_DENOM),
        min_mint_price: init_msg.mint_price.clone(),
        mint_fee_bps: 10000,
        max_trading_offset_secs: 60 * 60 * 24 * 7,
        extension: ParamsExtension {
            max_token_limit: 10,
            max_per_address_limit: 10,
            airdrop_mint_fee_bps: 100,
            dev_fee_address: DEV_ADDRESS.to_string(),
            airdrop_mint_price: params_extension.airdrop_mint_price.clone(),
        },
    };
    let vt =
        open_edition_minter_ibc_template(params_extension, init_msg, custom_minter_params).unwrap();
    let (mut router, creator, buyer) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();

    // give the buyer some of the IBC asset
    router
        .sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: buyer.to_string(),
                amount: vec![mint_price.clone()],
            }
        }))
        .map_err(|err| println!("{err:?}"))
        .ok();

    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 100, None);

    // Mint succeeds
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(buyer.clone(), minter_addr, &mint_msg, &[mint_price.clone()]);
    assert!(res.is_ok());

    // confirm balances
    // confirm buyer IBC assets spent
    let balance = router.wrap().query_balance(buyer, denom).unwrap();
    assert_eq!(balance.amount, Uint128::zero());
    // for noble, seller has 0% IBC asset
    let balance = router.wrap().query_balance(creator, denom).unwrap();
    assert_eq!(balance.amount, Uint128::zero());
    // confirm mint_price 50% sent to community pool, 50% sent to dev
    // "community_pool" address from packages/sg-multi-test/src/multi.rs
    let balance = router
        .wrap()
        .query_balance(Addr::unchecked(FOUNDATION), denom)
        .unwrap();
    assert_eq!(balance.amount, mint_price.amount * Decimal::percent(50));
}

#[test]
fn zero_mint_fee() {
    // factory needs airdrop_mint_price: 0
    // factory needs mint_fee_bps: 0 (0%)

    // allow ibc/frenz denom
    let denom = "ibc/frenz";
    let mint_price = coin(MIN_MINT_PRICE_OPEN_EDITION, denom.to_string());

    let params_extension = ParamsExtension {
        max_token_limit: 10,
        max_per_address_limit: 10,
        airdrop_mint_fee_bps: 100,
        airdrop_mint_price: Coin {
            denom: denom.to_string(),
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
        Some(mint_price.clone()),
    );

    let custom_minter_params = OpenEditionMinterParams {
        code_id: 1,
        allowed_sg721_code_ids: vec![1, 3, 5, 6],
        frozen: false,
        creation_fee: coin(CREATION_FEE, NATIVE_DENOM),
        min_mint_price: init_msg.mint_price.clone(),
        mint_fee_bps: 0,
        max_trading_offset_secs: 60 * 60 * 24 * 7,
        extension: ParamsExtension {
            max_token_limit: 10,
            max_per_address_limit: 10,
            airdrop_mint_fee_bps: 100,
            dev_fee_address: DEV_ADDRESS.to_string(),
            airdrop_mint_price: params_extension.airdrop_mint_price.clone(),
        },
    };
    let vt =
        open_edition_minter_ibc_template(params_extension, init_msg, custom_minter_params).unwrap();
    let (mut router, _, buyer) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();

    // give the buyer some of the IBC asset
    router
        .sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: buyer.to_string(),
                amount: vec![mint_price.clone()],
            }
        }))
        .map_err(|err| println!("{err:?}"))
        .ok();

    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 100, None);

    // Mint succeeds
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(buyer, minter_addr, &mint_msg, &[mint_price]);
    assert!(res.is_ok());
}
