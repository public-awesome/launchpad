use crate::claim_airdrop::claim_airdrop;
#[cfg(not(feature = "library"))]
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::CONFIG;

use cosmwasm_std::entry_point;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;
use sg1::fair_burn;

use build_message::{state_config, whitelist_instantiate};
use validation::validate_instantiation_params;

const CONTRACT_NAME: &str = "crates.io:sg-eth-airdrop";
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
    fair_burn(
        env.contract.address.to_string(),
        INSTANTIATION_FEE,
        None,
        &mut res,
    );
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
        ExecuteMsg::ClaimAirdrop {
            eth_address,
            eth_sig,
        } => claim_airdrop(deps, info, _env, eth_address, eth_sig),
    }
}

mod build_message {
    use super::*;
    use crate::state::Config;
    use cosmwasm_std::{to_json_binary, Deps, SubMsg, WasmMsg};

    use validation::validate_airdrop_amount;
    use whitelist_immutable::msg::InstantiateMsg as WGInstantiateMsg;

    pub const GENERIC_WHITELIST_LABEL: &str = "Generic Whitelist for Airdrop";
    pub const INIT_WHITELIST_REPLY_ID: u64 = 1;

    pub fn whitelist_instantiate(env: Env, msg: InstantiateMsg) -> Result<SubMsg, ContractError> {
        let whitelist_instantiate_msg = WGInstantiateMsg {
            addresses: msg.addresses,
            mint_discount_bps: Some(0),
            per_address_limit: msg.per_address_limit,
        };
        let wasm_msg = WasmMsg::Instantiate {
            code_id: msg.whitelist_code_id,
            admin: Some(env.contract.address.to_string()),
            funds: vec![],
            label: GENERIC_WHITELIST_LABEL.to_string(),
            msg: to_json_binary(&whitelist_instantiate_msg)?,
        };
        Ok(SubMsg::reply_on_success(wasm_msg, INIT_WHITELIST_REPLY_ID))
    }

    pub fn state_config(
        deps: Deps,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Config, ContractError> {
        Ok(Config {
            admin: info.sender,
            claim_msg_plaintext: msg.clone().claim_msg_plaintext,
            airdrop_amount: validate_airdrop_amount(msg.airdrop_amount)?,
            whitelist_address: None,
            minter_address: deps.api.addr_validate(msg.minter_address.as_ref())?,
        })
    }
}

mod validation {
    use super::*;
    use cosmwasm_std::Uint128;
    use cw_utils::must_pay;
    use sg_std::NATIVE_DENOM;

    const MIN_AIRDROP: u128 = 10_000_000; // 10 STARS
    const MAX_AIRDROP: u128 = 100_000_000_000_000; // 100 million STARS

    pub fn validate_instantiation_params(
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<(), ContractError> {
        validate_airdrop_amount(msg.airdrop_amount)?;
        validate_plaintext_msg(msg.claim_msg_plaintext)?;
        validate_instantiate_funds(info)?;
        Ok(())
    }

    pub fn validate_instantiate_funds(info: MessageInfo) -> Result<(), ContractError> {
        let amount = must_pay(&info, NATIVE_DENOM)?;
        if amount < Uint128::from(INSTANTIATION_FEE) {
            return Err(ContractError::InsufficientFundsInstantiate {});
        };
        Ok(())
    }

    pub fn validate_airdrop_amount(airdrop_amount: u128) -> Result<u128, ContractError> {
        if airdrop_amount < MIN_AIRDROP {
            return Err(ContractError::AirdropTooSmall {});
        };
        if airdrop_amount > MAX_AIRDROP {
            return Err(ContractError::AirdropTooBig {});
        };
        Ok(airdrop_amount)
    }

    pub fn validate_plaintext_msg(plaintext_msg: String) -> Result<(), ContractError> {
        if !plaintext_msg.contains("{wallet}") {
            return Err(ContractError::PlaintextMsgNoWallet {});
        }
        if plaintext_msg.len() > 1000 {
            return Err(ContractError::PlaintextTooLong {});
        }
        Ok(())
    }
}
