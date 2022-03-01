pub mod fees;
mod msg;
mod query;
mod route;

pub const NATIVE_DENOM: &str = "ustars";
// 3/4/2022 16:00:00 ET 1646427600000000000
pub const GENESIS_MINT_START_TIME: u64 = 1645376400000000000;

pub use msg::{
    create_claim_for_msg, create_fund_community_pool_msg, ClaimAction, StargazeMsg,
    StargazeMsgWrapper,
};

pub use fees::burn_and_distribute_fee;
pub use query::StargazeQuery;
pub use route::StargazeRoute;

// This export is added to all contracts that import this package, signifying that they require
// "stargaze" support on the chain they run on.
#[no_mangle]
extern "C" fn requires_stargaze() {}
