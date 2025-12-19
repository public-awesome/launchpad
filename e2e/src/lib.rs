// Allow large error variants in e2e tests
#![allow(clippy::result_large_err)]

#[cfg(not(target_arch = "wasm32"))]
mod tests;

#[cfg(not(target_arch = "wasm32"))]
mod helpers;
