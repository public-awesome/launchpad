use cosmwasm_std::Addr;
use cw_multi_test::IntoAddr;

pub fn owner() -> Addr {
    "admin0001".into_addr()
}

pub fn stargaze_wallet_01() -> Addr {
    "stargaze_wallet_01".into_addr()
}

pub fn stargaze_wallet_02() -> Addr {
    "stargaze_wallet_02".into_addr()
}

pub fn mock_airdrop_addr() -> Addr {
    "contract4".into_addr()
}

pub const CONFIG_PLAINTEXT: &str = "My Stargaze address is {wallet} and I want a Winter Pal.";
pub const NATIVE_DENOM: &str = "ugas";
