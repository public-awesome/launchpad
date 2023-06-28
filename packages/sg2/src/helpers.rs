use crate::error::ContractError;
use crate::Token;
use cosmwasm_std::{StdError, Uint128};

pub fn get_denom(token_type: Token) -> Result<String, ContractError> {
    let denom = match token_type {
        Token::Fungible(coin) => coin.denom,
        Token::NonFungible(_) => return Err(ContractError::IncorrectFungibility {}),
    };
    Ok(denom)
}

pub fn get_amount(token: Token) -> Result<Uint128, ContractError> {
    let amount = match token {
        Token::Fungible(coin) => coin.amount,
        Token::NonFungible(_) => return Err(ContractError::IncorrectFungibility {}),
    };
    Ok(amount)
}

pub fn get_amount_std_error(token: Token) -> Result<Uint128, StdError> {
    let amount = get_amount(token);
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

pub fn get_denom_std_error(token_type: Token) -> Result<String, StdError> {
    let denom = get_denom(token_type);
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
