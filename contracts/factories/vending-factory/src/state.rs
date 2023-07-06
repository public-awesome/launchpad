use cosmwasm_schema::cw_serde;
use cosmwasm_std::Coin;
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
pub type VendingMinterParams = MinterParams<ParamsExtension>;

pub const SUDO_PARAMS: Item<VendingMinterParams> = Item::new("sudo-params");

// migration version constants
pub const EARLIEST_VERSION: &str = "2.1.0";
pub const TO_VERSION: &str = "3.0.0";
