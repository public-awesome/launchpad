mod msg;
mod params;
mod query;
mod route;

pub const NATIVE_DENOM: &str = "ustars";
// 3/11/2022 16:00:00 ET
pub const GENESIS_MINT_START_TIME: u64 = 1647032400000000000;

pub use msg::{
    create_claim_for_msg, create_fund_community_pool_msg, ClaimAction, StargazeMsg,
    StargazeMsgWrapper,
};

pub type Response = cosmwasm_std::Response<StargazeMsgWrapper>;
pub type SubMsg = cosmwasm_std::SubMsg<StargazeMsgWrapper>;
pub type CosmosMsg = cosmwasm_std::CosmosMsg<StargazeMsgWrapper>;

pub use params::query_param_u32;
pub use query::StargazeQuery;
pub use route::StargazeRoute;

// This export is added to all contracts that import this package, signifying that they require
// "stargaze" support on the chain they run on.
#[no_mangle]
extern "C" fn requires_stargaze() {}
