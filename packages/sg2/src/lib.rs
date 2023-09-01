use cosmwasm_schema::cw_serde;
use cosmwasm_std::{coin, Coin, StdError, Uint128};

pub mod error;
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

impl Token {
    pub fn new_fungible_token(amount: u128, denom: String) -> Token {
        Token::Fungible(coin(amount, denom))
    }

    pub fn denom(self) -> Result<String, StdError> {
        let denom = match self {
            Token::Fungible(coin) => coin.denom,
            Token::NonFungible(_) => {
                return Err(StdError::generic_err("non-fungible tokens have no denom"))
            }
        };
        Ok(denom)
    }

    /// A nice helper that can be used to check if its fungible or not
    pub fn is_fungible(self) -> bool {
        match self {
            Token::Fungible(_) => true,
            Token::NonFungible(_) => false,
        }
    }

    pub fn amount(self) -> Result<Uint128, StdError> {
        let amount = match self {
            Token::Fungible(coin) => coin.amount,
            Token::NonFungible(_) => {
                return Err(StdError::generic_err("non-fungible tokens have no amount"))
            }
        };
        Ok(amount)
    }
}

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
