use cosmwasm_schema::cw_serde;
use cosmwasm_std::Coin;

pub mod msg;
pub mod query;
pub mod tests;

pub type CodeId = u64;

// Sane defaults used for migrating v2.1.0 factory to v3.0.0 factory
pub const DEFAULT_MAX_ROYALTY_BPS: u64 = 1000;
pub const DEFAULT_MAX_ROYALTY_INCREASE_RATE_BPS: u64 = 200;

/// Common params for all minters used for storage
#[cw_serde]
pub struct MinterParams<T> {
    /// The minter code id
    pub code_id: u64,
    pub allowed_sg721_code_ids: Vec<CodeId>,
    pub frozen: bool,
    pub creation_fee: Coin,
    pub min_mint_price: Coin,
    pub mint_fee_bps: u64,
    pub max_trading_offset_secs: u64,
    pub max_royalty_bps: Option<u64>,
    pub max_royalty_increase_rate_bps: Option<u64>,
    pub extension: T,
}
