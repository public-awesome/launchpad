use cosmwasm_schema::cw_serde;
use cosmwasm_std::Coin;

pub mod msg;
pub mod query;
pub mod tests;

pub type CodeId = u64;

pub use Token::{Fungible, NonFungible};

#[cw_serde]
pub enum Token {
    Fungible(Coin),
    NonFungible(String),
}

// #[cw_serde]
// pub struct Token {
//     pub thing: String,
// }

/// Common params for all minters used for storage
#[cw_serde]
pub struct MinterParams<T> {
    /// The minter code id
    pub code_id: u64,
    pub allowed_sg721_code_ids: Vec<CodeId>,
    pub frozen: bool,
    pub creation_fee: Coin,
    pub min_mint_price: Token,
    pub mint_fee_bps: u64,
    pub max_trading_offset_secs: u64,
    pub extension: T,
}
