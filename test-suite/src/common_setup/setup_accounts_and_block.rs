use cosmwasm_std::{coins, Addr, Coin, Timestamp, Uint128};
use cw4::Member;
use cw_multi_test::{BankSudo, Executor, SudoMsg};
use sg_std::NATIVE_DENOM;

use crate::common_setup::contract_boxes::{contract_group, App};
use crate::common_setup::setup_minter::common::constants::DEV_ADDRESS;

const OWNER: &str = "admin0001";

pub const CREATION_FEE: u128 = 5_000_000_000;
pub const INITIAL_BALANCE: u128 = 2_000_000_000;

// uploads code and returns address of group contract
pub fn instantiate_group(app: &mut App, members: Vec<Member>) -> Addr {
    let group_id = app.store_code(contract_group());
    println!("group_id: {group_id}");
    let msg = cw4_group::msg::InstantiateMsg {
        admin: Some(OWNER.into()),
        members,
    };
    app.instantiate_contract(group_id, Addr::unchecked(OWNER), &msg, &[], "group", None)
        .unwrap()
}

// Add a creator account with initial balances
pub fn setup_accounts(router: &mut App) -> (Addr, Addr) {
    let buyer = Addr::unchecked("buyer");
    let creator = Addr::unchecked("creator");
    let dev = Addr::unchecked(DEV_ADDRESS);
    // 3,000 tokens
    let creator_funds = coins(INITIAL_BALANCE + CREATION_FEE, NATIVE_DENOM);
    // 2,000 tokens
    let buyer_funds = coins(INITIAL_BALANCE, NATIVE_DENOM);
    // 2,000 tokens
    let dev_funds = coins(INITIAL_BALANCE, NATIVE_DENOM);
    router
        .sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: creator.to_string(),
                amount: creator_funds.clone(),
            }
        }))
        .map_err(|err| println!("{err:?}"))
        .ok();

    router
        .sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: buyer.to_string(),
                amount: buyer_funds.clone(),
            }
        }))
        .map_err(|err| println!("{err:?}"))
        .ok();

    router
        .sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: dev.to_string(),
                amount: dev_funds,
            }
        }))
        .map_err(|err| println!("{err:?}"))
        .ok();

    // Check native balances
    let creator_native_balances = router.wrap().query_all_balances(creator.clone()).unwrap();
    assert_eq!(creator_native_balances, creator_funds);

    // Check native balances
    let buyer_native_balances = router.wrap().query_all_balances(buyer.clone()).unwrap();
    assert_eq!(buyer_native_balances, buyer_funds);

    // Check native balances
    let dev_native_balances = router.wrap().query_all_balances(dev).unwrap();
    assert_eq!(dev_native_balances, buyer_funds);

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
