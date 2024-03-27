use crate::claim_airdrop::claim_airdrop;
#[cfg(not(feature = "library"))]
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{AIRDROP_COUNT, CONFIG};

use cosmwasm_std::entry_point;
use cosmwasm_std::{DepsMut, Env, MessageInfo};
use cw2::set_contract_version;
use sg1::fair_burn;
use sg_std::Response;

use crate::build_messages::{state_config, whitelist_instantiate};
use crate::register::register;
use crate::validation::validate_instantiation_params;

const CONTRACT_NAME: &str = "crates.io:dydx-airdrop";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const INSTANTIATION_FEE: u128 = 100_000_000; // 100 STARS

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    validate_instantiation_params(info.clone(), msg.clone())?;
    let mut res = Response::new();
    fair_burn(INSTANTIATION_FEE, None, &mut res);
    AIRDROP_COUNT.save(deps.storage, &0)?;
    let cfg = state_config(deps.as_ref(), info.clone(), msg.clone())?;
    CONFIG.save(deps.storage, &cfg)?;
    Ok(res
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION)
        .add_attribute("sender", info.sender)
        .add_submessage(whitelist_instantiate(env, msg)?))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Register {
            eth_address,
            eth_sig,
        } => register(deps, info, _env, eth_address, eth_sig),
        ExecuteMsg::ClaimAirdrop {
            eth_address,
            eth_sig,
        } => claim_airdrop(deps, info, _env, eth_address, eth_sig),
    }
}
