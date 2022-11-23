use super::{check_instantiate_funds, INSTANTIATION_FEE};
use crate::ContractError;
use cosmwasm_std::MessageInfo;
use sg1::fair_burn;
use sg_std::Response;

pub fn check_funds_and_fair_burn(info: MessageInfo) -> Result<Response, ContractError> {
    check_instantiate_funds(info)?;
    let mut res = Response::new();
    fair_burn(INSTANTIATION_FEE, None, &mut res);
    Ok(res)
}
