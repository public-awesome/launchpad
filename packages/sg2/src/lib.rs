use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, StdError, Uint128};

pub mod error;
pub mod msg;
pub mod query;
pub mod tests;

pub type CodeId = u64;

use error::ContractError;
pub use Token::{Fungible, NonFungible};

#[cw_serde]
pub enum Token {
    Fungible(Coin),
    NonFungible(String),
}

impl Token {
    pub fn get_denom(self) -> Result<String, ContractError> {
        let denom = match self {
            Token::Fungible(coin) => coin.denom,
            Token::NonFungible(_) => return Err(ContractError::IncorrectFungibility {}),
        };
        Ok(denom)
    }

    pub fn get_amount(self) -> Result<Uint128, ContractError> {
        let amount = match self {
            Token::Fungible(coin) => coin.amount,
            Token::NonFungible(_) => return Err(ContractError::IncorrectFungibility {}),
        };
        Ok(amount)
    }

    pub fn get_amount_std_error(self) -> Result<Uint128, StdError> {
        let amount = self.get_amount();
        let fungibility_error = "Incorrect Fungibility".to_string();
        let result = match amount {
            Ok(token_amount) => token_amount,
            Err(_) => {
                return Err(StdError::GenericErr {
                    msg: fungibility_error,
                })
            }
        };
        Ok(result)
    }

    pub fn get_denom_std_error(self) -> Result<String, StdError> {
        let denom = self.get_denom();
        let fungibility_error = "Incorrect Fungibility".to_string();
        let result = match denom {
            Ok(denom_result) => denom_result,
            Err(_) => {
                return Err(StdError::GenericErr {
                    msg: fungibility_error,
                })
            }
        };
        Ok(result)
    }
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
