pub mod contract;
mod error;

#[cfg(test)]
#[path = "./tests/integration_tests.rs"]
mod integration_tests;
#[cfg(test)]
#[path = "./tests/collection_whitelist_tests.rs"]
mod collection_whitelist_tests;
pub mod msg;

#[path = "./helpers/build_msg.rs"]
pub mod build_msg;
#[path = "./helpers/computation.rs"]
pub mod computation;
#[path = "./helpers/constants.rs"]
pub mod constants;
#[path = "./helpers/responses.rs"]
pub mod responses;
pub mod state;

pub use crate::error::ContractError;
#[path = "../ethereum/ethereum.rs"]
mod ethereum;
#[path = "../ethereum/signature_verify.rs"]
mod signature_verify;
