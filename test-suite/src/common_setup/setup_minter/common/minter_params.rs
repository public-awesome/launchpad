use cosmwasm_std::{Addr, Timestamp};
use vending_factory::msg::VendingMinterInitMsgExtension;

use crate::common_setup::msg::MinterInstantiateParams;

pub fn minter_params_all(
    num_tokens: u32,
    splits_addr: Option<String>,
    start_time: Option<Timestamp>,
    init_msg: Option<VendingMinterInitMsgExtension>,
    allowed_burn_collections: Option<Vec<Addr>>,
) -> MinterInstantiateParams {
    MinterInstantiateParams {
        num_tokens,
        splits_addr,
        start_time,
        init_msg,
        allowed_burn_collections,
    }
}

pub fn minter_params_token(num_tokens: u32) -> MinterInstantiateParams {
    MinterInstantiateParams {
        num_tokens,
        splits_addr: None,
        start_time: None,
        init_msg: None,
        allowed_burn_collections: None,
    }
}

pub fn minter_params_allowed_burn_collections(
    num_tokens: u32,
    allowed_burn_collections: Vec<Addr>,
) -> MinterInstantiateParams {
    MinterInstantiateParams {
        num_tokens,
        splits_addr: None,
        start_time: None,
        init_msg: None,
        allowed_burn_collections: Some(allowed_burn_collections),
    }
}
