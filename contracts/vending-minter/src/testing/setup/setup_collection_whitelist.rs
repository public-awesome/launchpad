use cosmwasm_std::{coin, Addr, Timestamp};
use cw_multi_test::Executor;
use sg_multi_test::StargazeApp;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
use sg_whitelist::msg::InstantiateMsg as WhitelistInstantiateMsg;

use crate::testing::setup::contract_boxes::contract_whitelist;

const WHITELIST_AMOUNT: u128 = 66_000_000;
const WL_PER_ADDRESS_LIMIT: u32 = 1;

pub fn setup_whitelist_contract(router: &mut StargazeApp, creator: &Addr) -> Addr {
    let whitelist_code_id = router.store_code(contract_whitelist());

    let msg = WhitelistInstantiateMsg {
        members: vec![],
        start_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME + 100),
        end_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10000000),
        mint_price: coin(WHITELIST_AMOUNT, NATIVE_DENOM),
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