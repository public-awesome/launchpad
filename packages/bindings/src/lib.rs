mod msg;
mod route;

pub use route::StargazeRoute;

pub use msg::{create_claim_msg, create_fund_community_pool_msg, StargazeMsgWrapper};

// This export is added to all contracts that import this package, signifying that they require
// "stargaze" support on the chain they run on.
#[no_mangle]
extern "C" fn requires_stargaze() {}
