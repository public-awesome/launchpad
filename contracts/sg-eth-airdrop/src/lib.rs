pub mod contract;
mod error;
pub mod msg;
pub mod query;
pub mod reply;
pub mod state;
pub use crate::error::ContractError;
mod claim_airdrop;

#[cfg(test)]
#[path = "./testing/lib.rs"]
pub mod tests_folder;
