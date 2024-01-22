use cosmwasm_std::{coin, Addr, Timestamp};
use cw_multi_test::Executor;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
use sg_whitelist::msg::InstantiateMsg as WhitelistInstantiateMsg;

use crate::common_setup::{
    contract_boxes::{contract_collection_whitelist, App},
    setup_accounts_and_block::setup_block_time,
};

pub const WHITELIST_AMOUNT: u128 = 66_000_000;
const ZERO_FEE_WHITELIST: u128 = 0;
const WL_PER_ADDRESS_LIMIT: u32 = 1;

pub fn setup_whitelist_contract(
    router: &mut App,
    creator: &Addr,
    whitelist_code_id: Option<u64>,
    denom: Option<&str>,
) -> Addr {
    let whitelist_code_id = match whitelist_code_id {
        Some(value) => value,
        None => router.store_code(contract_collection_whitelist()),
    };
    let denom = match denom {
        Some(value) => value,
        None => NATIVE_DENOM,
    };

    let msg = WhitelistInstantiateMsg {
        members: vec![],
        start_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME + 100),
        end_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10000000),
        mint_price: coin(WHITELIST_AMOUNT, denom),
        per_address_limit: WL_PER_ADDRESS_LIMIT,
        member_limit: 1000,
        admins: vec![creator.to_string()],
        admins_mutable: true,
    };
    router
        .instantiate_contract(
            whitelist_code_id,
            creator.clone(),
            &msg,
            &[coin(100_000_000, NATIVE_DENOM)],
            "whitelist",
            None,
        )
        .unwrap()
}

pub fn setup_zero_fee_whitelist_contract(
    router: &mut App,
    creator: &Addr,
    whitelist_code_id: Option<u64>,
) -> Addr {
    let whitelist_code_id = match whitelist_code_id {
        Some(value) => value,
        None => router.store_code(contract_collection_whitelist()),
    };

    let msg = WhitelistInstantiateMsg {
        members: vec![],
        start_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME + 100),
        end_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10000000),
        mint_price: coin(ZERO_FEE_WHITELIST, NATIVE_DENOM),
        per_address_limit: WL_PER_ADDRESS_LIMIT,
        member_limit: 1000,
        admins: vec![creator.to_string()],
        admins_mutable: true,
    };

    router
        .instantiate_contract(
            whitelist_code_id,
            creator.to_owned(),
            &msg,
            &[coin(100_000_000, NATIVE_DENOM)],
            "whitelist",
            None,
        )
        .unwrap()
}

pub fn configure_collection_whitelist(
    router: &mut App,
    creator: Addr,
    buyer: Addr,
    minter_addr: Addr,
) -> Addr {
    let whitelist_addr = setup_whitelist_contract(router, &creator, None, None);
    const AFTER_GENESIS_TIME: Timestamp = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 100);

    // Set to just before genesis mint start time
    setup_block_time(router, GENESIS_MINT_START_TIME - 10, None);

    // Update whitelist_expiration fails if not admin
    let wl_msg = sg_whitelist::msg::ExecuteMsg::UpdateEndTime(AFTER_GENESIS_TIME);
    router
        .execute_contract(buyer, whitelist_addr.clone(), &wl_msg, &[])
        .unwrap_err();

    // Update whitelist_expiration succeeds when from admin
    let wl_msg = sg_whitelist::msg::ExecuteMsg::UpdateEndTime(AFTER_GENESIS_TIME);
    let res = router.execute_contract(creator.clone(), whitelist_addr.clone(), &wl_msg, &[]);
    assert!(res.is_ok());

    let wl_msg = sg_whitelist::msg::ExecuteMsg::UpdateStartTime(Timestamp::from_nanos(0));
    let res = router.execute_contract(creator.clone(), whitelist_addr.clone(), &wl_msg, &[]);
    assert!(res.is_ok());

    // Set whitelist in minter contract
    let set_whitelist_msg = vending_minter::msg::ExecuteMsg::SetWhitelist {
        whitelist: whitelist_addr.to_string(),
    };
    let res = router.execute_contract(creator, minter_addr, &set_whitelist_msg, &[]);
    assert!(res.is_ok());
    whitelist_addr
}
