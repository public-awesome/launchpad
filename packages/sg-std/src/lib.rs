mod msg;
mod route;

pub const NATIVE_DENOM: &str = "ustars";
// 3/1/2022 12:00:00 ET
pub const GENESIS_MINT_START_TIME: u64 = 1646154000000000000;

pub use msg::{
    create_claim_for_msg, create_fund_community_pool_msg, ClaimAction, StargazeMsg,
    StargazeMsgWrapper,
};
pub use route::StargazeRoute;

// This export is added to all contracts that import this package, signifying that they require
// "stargaze" support on the chain they run on.
#[no_mangle]
extern "C" fn requires_stargaze() {}
