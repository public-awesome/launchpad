use super::check_instantiate_funds;
use crate::ContractError;
use cosmwasm_std::MessageInfo;
use sg1::fair_burn;
use sg_std::Response;

pub fn check_funds_and_fair_burn(info: MessageInfo) -> Result<Response, ContractError> {
    let amount = check_instantiate_funds(info)?;
    let mut res = Response::new();
    fair_burn(amount, None, &mut res);
    Ok(res)
}
