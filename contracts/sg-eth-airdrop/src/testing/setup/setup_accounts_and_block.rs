use crate::testing::constants::{CREATION_FEE, INITIAL_BALANCE, NATIVE_DENOM};
use cosmwasm_std::{coins, Addr, Timestamp};
use cw_multi_test::{BankSudo, SudoMsg};
use sg_multi_test::StargazeApp;

// Set blockchain time to after mint by default
pub fn setup_block_time(router: &mut StargazeApp, nanos: u64, height: Option<u64>) {
    let mut block = router.block_info();
    block.time = Timestamp::from_nanos(nanos);
    if let Some(h) = height {
        block.height = h;
    }
    router.set_block(block);
}

// Add a creator account with initial balances
pub fn setup_accounts(router: &mut StargazeApp) -> (Addr, Addr) {
    let buyer = Addr::unchecked("buyer");
    let creator = Addr::unchecked("creator");
    // 3,000 tokens
    let creator_funds = coins(INITIAL_BALANCE + CREATION_FEE, NATIVE_DENOM);
    // 2,000 tokens
    let buyer_funds = coins(INITIAL_BALANCE, NATIVE_DENOM);
    router
        .sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: creator.to_string(),
                amount: creator_funds.clone(),
            }
        }))
        .map_err(|err| println!("{:?}", err))
        .ok();

    router
        .sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: buyer.to_string(),
                amount: buyer_funds.clone(),
            }
        }))
        .map_err(|err| println!("{:?}", err))
        .ok();

    // Check native balances
    let creator_native_balances = router.wrap().query_all_balances(creator.clone()).unwrap();
    assert_eq!(creator_native_balances, creator_funds);

    // Check native balances
    let buyer_native_balances = router.wrap().query_all_balances(buyer.clone()).unwrap();
    assert_eq!(buyer_native_balances, buyer_funds);

    (creator, buyer)
}
