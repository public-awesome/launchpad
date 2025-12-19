use cosmwasm_std::{coins, Addr, Coin, Timestamp, Uint128};
use cw4::Member;
use cw_multi_test::{BankSudo, Executor, IntoAddr, SudoMsg};

use sg_utils::{FEE_DENOM, NATIVE_DENOM};

use crate::common_setup::contract_boxes::{contract_group, App};
use crate::common_setup::setup_minter::common::constants::dev_address;

const OWNER: &str = "admin0001";

pub const CREATION_FEE: u128 = 5_000_000_000;
pub const INITIAL_BALANCE: u128 = 2_000_000_000;

// uploads code and returns address of group contract
pub fn instantiate_group(app: &mut App, members: Vec<Member>) -> Addr {
    let group_id = app.store_code(contract_group());
    println!("group_id: {group_id}");
    let owner_addr = OWNER.into_addr();
    let msg = cw4_group::msg::InstantiateMsg {
        admin: Some(owner_addr.to_string()),
        members,
    };
    app.instantiate_contract(group_id, owner_addr, &msg, &[], "group", None)
        .unwrap()
}

// Add a creator account with initial balances
pub fn setup_accounts(router: &mut App) -> (Addr, Addr) {
    // Use IntoAddr to create valid bech32 addresses
    let buyer = "buyer".into_addr();
    let creator = "creator".into_addr();
    let dev = dev_address();
    // 3,000 tokens in native denom
    let creator_native_funds = coins(INITIAL_BALANCE + CREATION_FEE, NATIVE_DENOM);
    // Also mint FEE_DENOM for minting operations (base-minter requires FEE_DENOM)
    let creator_fee_funds = coins(INITIAL_BALANCE + CREATION_FEE, FEE_DENOM);
    // 2,000 tokens
    let buyer_native_funds = coins(INITIAL_BALANCE, NATIVE_DENOM);
    let buyer_fee_funds = coins(INITIAL_BALANCE, FEE_DENOM);
    // 2,000 tokens
    let dev_native_funds = coins(INITIAL_BALANCE, NATIVE_DENOM);
    let dev_fee_funds = coins(INITIAL_BALANCE, FEE_DENOM);

    // Mint NATIVE_DENOM for all accounts
    router
        .sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: creator.to_string(),
                amount: creator_native_funds.clone(),
            }
        }))
        .map_err(|err| println!("{err:?}"))
        .ok();

    router
        .sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: buyer.to_string(),
                amount: buyer_native_funds.clone(),
            }
        }))
        .map_err(|err| println!("{err:?}"))
        .ok();

    router
        .sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: dev.to_string(),
                amount: dev_native_funds,
            }
        }))
        .map_err(|err| println!("{err:?}"))
        .ok();

    // Mint FEE_DENOM for all accounts (required by base-minter and other contracts)
    router
        .sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: creator.to_string(),
                amount: creator_fee_funds,
            }
        }))
        .map_err(|err| println!("{err:?}"))
        .ok();

    router
        .sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: buyer.to_string(),
                amount: buyer_fee_funds,
            }
        }))
        .map_err(|err| println!("{err:?}"))
        .ok();

    router
        .sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: dev.to_string(),
                amount: dev_fee_funds,
            }
        }))
        .map_err(|err| println!("{err:?}"))
        .ok();

    // Check native balances
    let creator_native_balances = router.wrap().query_all_balances(creator.clone()).unwrap();
    assert!(creator_native_balances.len() >= 1);

    // Check native balances
    let buyer_native_balances = router.wrap().query_all_balances(buyer.clone()).unwrap();
    assert!(buyer_native_balances.len() >= 1);

    (creator, buyer)
}

// Set blockchain time to after mint by default
pub fn setup_block_time(router: &mut App, nanos: u64, height: Option<u64>) {
    let mut block = router.block_info();
    block.time = Timestamp::from_nanos(nanos);
    if let Some(h) = height {
        block.height = h;
    }
    router.set_block(block);
}

// Deal with zero and non-zero coin amounts for msgs
pub fn coins_for_msg(msg_coin: Coin) -> Vec<Coin> {
    if msg_coin.amount > Uint128::zero() {
        vec![msg_coin]
    } else {
        vec![]
    }
}
