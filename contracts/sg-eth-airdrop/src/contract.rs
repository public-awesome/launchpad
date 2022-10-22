
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, StdResult};
use cw2::set_contract_version;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::error::ContractError;
use crate::msg::{EligibleResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG};
use sg_std::{Response};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg-eth-airdrop";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let cfg = Config {
        config: "test".to_string(),
    };
    CONFIG.save(deps.storage, &cfg)?;
    let res = Response::new();
    Ok(res
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION)
        .add_attribute("sender", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ClaimAirdrop {
            eth_address,
            eth_sig,
            stargaze_address,
            stargaze_sig,
        } => claim_airdrop(deps, eth_address, eth_sig, stargaze_address, stargaze_sig),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::AirdropEligible { eth_address } => {
            to_binary(&airdrop_check_eligible(deps, env, eth_address)?)
        }
    }
}

fn claim_airdrop(
    deps: DepsMut,
    eth_address: String,
    eth_sig: String,
    stargaze_address: String,
    stargaze_sig: String,
) -> Result<Response, ContractError> {
    let amount: u32 = 30000;
    let result: bool = true;
    let minter_page: String = "http://levana_page/airdrop".to_string();
    Ok(Response::new()
        .add_attribute("amount", amount.to_string())
        .add_attribute("result", result.to_string())
        .add_attribute("minter_page", minter_page))
}

fn airdrop_check_eligible(deps: Deps, env: Env, eth_address: String) -> StdResult<EligibleResponse> {
    let eligible = true;
    Ok(EligibleResponse { eligible })
}

// internal function only
fn airdrop_check_valid(deps: Deps, env: Env, msg: QueryMsg) -> bool {
    true
}
