use cosmwasm_std::Addr;
use cw_multi_test::IntoAddr;

// pub const LISTING_FEE: u128 = 0;
pub const CREATION_FEE: u128 = 5_000_000_000;
pub const MINT_PRICE: u128 = 100_000_000;

pub const MAX_TOKEN_LIMIT: u32 = 10000;

pub const MIN_MINT_PRICE: u128 = 50_000_000;
pub const AIRDROP_MINT_PRICE: u128 = 0;
pub const MINT_FEE_FAIR_BURN: u64 = 1_000; // 10%
pub const AIRDROP_MINT_FEE_FAIR_BURN: u64 = 10_000; // 100%
pub const SHUFFLE_FEE: u128 = 500_000_000;
pub const MAX_PER_ADDRESS_LIMIT: u32 = 50;
pub const MIN_MINT_PRICE_OPEN_EDITION: u128 = 100_000_000;

// Address functions - generate valid bech32 addresses for tests
pub fn dev_address() -> Addr {
    "dev0001".into_addr()
}

pub fn foundation_address() -> Addr {
    "foundation0001".into_addr()
}

pub fn liquidity_dao_address() -> Addr {
    "liquiditydao0001".into_addr()
}

pub fn launchpad_dao_address() -> Addr {
    "launchpaddao0001".into_addr()
}

// Legacy constants - use function versions instead for tests
pub const DEV_ADDRESS: &str = "stars1abcd4kdla12mh86psg4y4h6hh05g2hmqoap350";
pub const FOUNDATION: &str = "init19fp5yt25cdkdjnzp4dc3gqp6dmsjdslwatg0v8";
pub const LIQUIDITY_DAO_ADDRESS: &str = "init19fp5yt25cdkdjnzp4dc3gqp6dmsjdslwatg0v8";
pub const LAUNCHPAD_DAO_ADDRESS: &str = "init19fp5yt25cdkdjnzp4dc3gqp6dmsjdslwatg0v8";
