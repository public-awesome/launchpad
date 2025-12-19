// Allow deprecated APIs used in tests (mock_info, query_all_balances)
#![allow(deprecated)]
// Allow length comparison to one
#![allow(clippy::len_zero)]
// Allow dead code in test modules
#![allow(dead_code)]
// Allow clippy style warnings in tests
#![allow(clippy::redundant_clone)]
#![allow(clippy::unnecessary_to_owned)]
#![allow(clippy::useless_vec)]
// Allow slice::from_ref suggestion (lint name varies by Rust version)
#![allow(clippy::cloned_ref_to_slice_refs)]

pub mod common_setup;

#[cfg(test)]
mod base_factory;
#[cfg(test)]
mod base_minter;
#[cfg(test)]
mod open_edition_factory;
#[cfg(test)]
mod open_edition_minter;
// Disabled: sg721_base has been replaced with cw721-migration
// #[cfg(test)]
// mod sg721_base;
#[cfg(test)]
mod sg_eth_airdrop;
#[cfg(test)]
mod splits;
#[cfg(test)]
mod vending_factory;
#[cfg(test)]
mod vending_minter;
#[cfg(test)]
mod whitelist;
#[cfg(test)]
mod whitelist_immutable;
#[cfg(test)]
mod whitelist_merkletree;
