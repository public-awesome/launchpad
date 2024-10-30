use cosmwasm_schema::cw_serde;
use cosmwasm_std::Coin;
use cw_storage_plus::Item;
use sg2::CodeId;

#[cw_serde]
pub struct TokenMergeMinterParams {
    pub code_id: u64,
    pub allowed_sg721_code_ids: Vec<CodeId>,
    pub frozen: bool,
    pub creation_fee: Coin,
    pub max_trading_offset_secs: u64,
    pub max_token_limit: u32,
    pub max_per_address_limit: u32,
    pub airdrop_mint_price: Coin,
    pub airdrop_mint_fee_bps: u64,
    pub shuffle_fee: Coin,
}

pub const SUDO_PARAMS: Item<TokenMergeMinterParams> = Item::new("sudo-params");
