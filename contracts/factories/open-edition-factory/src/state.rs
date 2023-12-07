use cosmwasm_schema::cw_serde;
use cosmwasm_std::Coin;
use cw_storage_plus::Item;

use sg2::MinterParams;

#[cw_serde]
pub struct ParamsExtension {
    pub max_token_limit: u32,
    pub max_per_address_limit: u32,
    pub airdrop_mint_fee_bps: u64,
    pub airdrop_mint_price: Coin,
    pub dev_fee_address: String,
}
pub type OpenEditionMinterParams = MinterParams<ParamsExtension>;

pub const SUDO_PARAMS: Item<OpenEditionMinterParams> = Item::new("sudo-params");
