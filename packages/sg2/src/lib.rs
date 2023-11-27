use core::fmt;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::Coin;

pub mod msg;
pub mod query;
pub mod tests;

pub type CodeId = u64;

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
    pub uri_scheme: Option<UriScheme>,
    pub extension: T,
}

#[cw_serde]
pub enum UriScheme {
    Https,
    Ipfs,
}

impl fmt::Display for UriScheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ipfs => write!(f, "ipfs"),
            Self::Https => write!(f, "https"),
        }
    }
}
