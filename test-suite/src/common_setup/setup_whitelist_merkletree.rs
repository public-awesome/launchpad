use cosmwasm_std::{coin, Addr, Timestamp};
use cw_multi_test::Executor;

use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
use whitelist_mtree::msg::InstantiateMsg as WhitelistInstantiateMsg;

use crate::common_setup::contract_boxes::{contract_whitelist_merkletree, App};

pub const WHITELIST_AMOUNT: u128 = 66_000_000;
const WL_PER_ADDRESS_LIMIT: u32 = 1;

pub fn setup_whitelist_mtree_contract(
    router: &mut App,
    creator: &Addr,
    whitelist_code_id: Option<u64>,
    denom: Option<&str>,
    merkle_root: String,
) -> Addr {
    let whitelist_code_id = match whitelist_code_id {
        Some(value) => value,
        None => router.store_code(contract_whitelist_merkletree()),
    };
    let denom = match denom {
        Some(value) => value,
        None => NATIVE_DENOM,
    };

    let msg = WhitelistInstantiateMsg {
        start_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME + 100),
        end_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10_000_000),
        mint_price: coin(WHITELIST_AMOUNT, denom),
        per_address_limit: WL_PER_ADDRESS_LIMIT,
        admins: vec![creator.to_string()],
        admins_mutable: true,
        merkle_root,
        merkle_tree_uri: None,
    };
    router
        .instantiate_contract(
            whitelist_code_id,
            creator.clone(),
            &msg,
            &[],
            "whitelist",
            None,
        )
        .unwrap()
}
