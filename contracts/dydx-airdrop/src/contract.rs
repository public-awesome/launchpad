use crate::claim_airdrop::claim_airdrop;
// #[cfg(not(feature = "library"))]
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{AIRDROP_COUNT, CONFIG};

use cosmwasm_std::{
    attr, ensure, entry_point, BankMsg, Coin, CosmosMsg, Empty, Event, StdError, Uint128,
};
use cosmwasm_std::{DepsMut, Env, MessageInfo};
use cw2::set_contract_version;
use semver::Version;
use sg1::fair_burn;
use sg_std::{Response, NATIVE_DENOM};

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
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Register {
            eth_address,
            eth_sig,
        } => register(deps, info, env, eth_address, eth_sig),
        ExecuteMsg::ClaimAirdrop {
            eth_address,
            eth_sig,
        } => claim_airdrop(deps, info, env, eth_address, eth_sig),
        ExecuteMsg::WithdrawRemaining { recipient, amount } => {
            withdraw_remaining(deps, info, env, recipient, amount)
        }
    }
}

pub fn withdraw_remaining(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    recipient: String,
    amount: Option<Uint128>,
) -> Result<Response, ContractError> {
    let contract_info = deps
        .querier
        .query_wasm_contract_info(env.contract.address.clone())?;
    ensure!(
        contract_info.admin.is_some() && contract_info.admin.unwrap() == info.sender,
        ContractError::Unauthorized {
            sender: info.sender
        }
    );

    let total_amount = deps
        .querier
        .query_balance(env.contract.address.clone(), NATIVE_DENOM)?
        .amount;

    let recipient_addr = deps.api.addr_validate(&recipient)?;

    let amount_to_withdraw = match amount {
        Some(amount) => {
            if amount > total_amount {
                return Err(ContractError::InsufficientFunds {
                    balance: total_amount,
                    amount,
                });
            }
            amount
        }
        None => total_amount,
    };

    let msg = BankMsg::Send {
        to_address: recipient_addr.to_string(),
        amount: vec![Coin {
            denom: NATIVE_DENOM.to_string(),
            amount: amount_to_withdraw,
        }],
    };

    let res = Response::new()
        .add_message(CosmosMsg::Bank(msg))
        .add_attributes(vec![
            attr("action", "withdraw_remaining"),
            attr("amount", amount_to_withdraw),
            attr("recipient", recipient_addr.to_string()),
        ]);
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: Empty) -> Result<Response, ContractError> {
    let current_version = cw2::get_contract_version(deps.storage)?;
    if current_version.contract != CONTRACT_NAME {
        return Err(StdError::generic_err("Cannot upgrade to a different contract").into());
    }
    let version: Version = current_version
        .version
        .parse()
        .map_err(|_| StdError::generic_err("Invalid contract version"))?;
    let new_version: Version = CONTRACT_VERSION
        .parse()
        .map_err(|_| StdError::generic_err("Invalid contract version"))?;

    if version > new_version {
        return Err(StdError::generic_err("Cannot upgrade to a previous contract version").into());
    }
    // if same version return
    if version == new_version {
        return Ok(Response::new());
    }

    // set new contract version
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let event = Event::new("migrate")
        .add_attribute("from_name", current_version.contract)
        .add_attribute("from_version", current_version.version)
        .add_attribute("to_name", CONTRACT_NAME)
        .add_attribute("to_version", CONTRACT_VERSION);
    Ok(Response::new().add_event(event))
}
