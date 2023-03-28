use crate::ContractError;
use crate::ContractError::CheckedMultiplyFractionError;
use cosmwasm_std::Uint128;

pub fn get_three_percent_of_tokens(num_tokens: u32) -> Result<Uint128, ContractError> {
    let three_percent = (Uint128::new(3), Uint128::new(100));
    let three_percent_tokens = Uint128::from(num_tokens)
        .checked_mul_ceil(three_percent)
        .map_err(|_| CheckedMultiplyFractionError {})?;
    Ok(three_percent_tokens)
}

// Check per address limit to make sure it's <= 1% num tokens
pub fn check_dynamic_per_address_limit(
    per_address_limit: u32,
    num_tokens: u32,
    max_per_address_limit: u32,
) -> Result<bool, ContractError> {
    if per_address_limit > max_per_address_limit {
        return Ok(false);
    }
    if num_tokens < 100 {
        return Ok(per_address_limit <= 3);
    }
    let three_percent_tokens = get_three_percent_of_tokens(num_tokens)?;
    let result = Uint128::from(per_address_limit) <= three_percent_tokens;
    Ok(result)
}
