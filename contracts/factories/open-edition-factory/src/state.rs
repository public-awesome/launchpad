use cosmwasm_schema::cw_serde;
use cosmwasm_std::Coin;
use cw_storage_plus::Item;

use sg2::MinterParams;

#[cw_serde]
pub struct ParamsExtension {
    pub token_id_prefix_length: u32,
    pub abs_max_mint_per_address: u32,
    pub airdrop_mint_fee_bps: u64,
    pub airdrop_mint_price: Coin,
    pub dev_fee_address: String,
    pub dev_fee_bps: u64,
}
pub type OpenEditionMinterParams = MinterParams<ParamsExtension>;

pub const SUDO_PARAMS: Item<OpenEditionMinterParams> = Item::new("sudo-params");