use cosmwasm_schema::cw_serde;
use cosmwasm_std::{coin, Coin};
use cw_storage_plus::Item;
use sg2::MinterParams;

/// Parameters common to all vending minters, as determined by governance
#[cw_serde]
pub struct ParamsExtension {
    pub max_token_limit: u32,
    pub max_per_address_limit: u32,
    pub airdrop_mint_price: Coin,
    pub airdrop_mint_fee_bps: u64,
    pub shuffle_fee: Coin,
}

#[cfg(test)]
impl Default for ParamsExtension {
    fn default() -> Self {
        Self {
            max_token_limit: 10000,
            max_per_address_limit: 50,
            airdrop_mint_price: coin(0, "uscrt"),
            airdrop_mint_fee_bps: 10000,
            shuffle_fee: coin(500_000_000, "uscrt"),
        }
    }
}

pub type VendingMinterParams = MinterParams<ParamsExtension>;

// impl Default for VendingMinterParams {
//     fn default() -> Self {
//         Self {
//             code_id: 1,
//             allowed_sg721_code_ids: vec![1, 3, 5, 6],
//             frozen: false,
//             creation_fee: coin(5_000_000_000, "uscrt"),
//             min_mint_price: coin(50_000_000, "uscrt"),
//             mint_fee_bps: 1000,
//             max_trading_offset_secs: 60 * 60 * 24 * 7,
//             extension: ParamsExtension::default(),
//         }
//     }
// }

pub const SUDO_PARAMS: Item<VendingMinterParams> = Item::new("sudo-params");
