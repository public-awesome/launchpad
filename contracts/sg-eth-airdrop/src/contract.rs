#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{EligibleResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
#[allow(unused_imports)]
use crate::signature_verify::{query_verify_cosmos, query_verify_ethereum_text};
use crate::state::{Config, CONFIG, ELIGIBLE_ETH_ADDRS};
use sg_std::Response;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg-eth-airdrop";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
#[allow(unused_variables)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let cfg = Config {
        admin: info.sender.clone(),
        claim_msg_plaintext: msg.config.claim_msg_plaintext,
        amount: msg.config.amount, 
        minter_page: msg.config.minter_page
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
#[allow(unused_variables)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ClaimAirdrop {
            eth_address,
            eth_sig
        } => claim_airdrop(deps, eth_address, eth_sig, info),
        ExecuteMsg::AddEligibleEth { eth_address } => {
            add_eligible_eth(deps, eth_address, info.sender)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
#[allow(unused_variables)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::AirdropEligible { eth_address } => {
            to_binary(&airdrop_check_eligible(deps, eth_address)?)
        }
    }
}

fn claim_airdrop(
    deps: DepsMut,
    eth_address: String,
    eth_sig: String,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let plaintext_msg = str::replace(
        &config.claim_msg_plaintext,
        "{wallet}",
        &info.sender.to_string(),
    );
    let is_eligible: EligibleResponse = airdrop_check_eligible(deps.as_ref(), eth_address.clone())?; 
    let eth_sig_hex = hex::decode(eth_sig).unwrap();
    let valid_eth_signature =
        query_verify_ethereum_text(deps.as_ref(), &plaintext_msg, &eth_sig_hex, &eth_address)
            .unwrap();

    if is_eligible.eligible && valid_eth_signature.verifies {
        remove_eth_address_from_eligible(deps, eth_address);
    }
    
    Ok(Response::new()
        .add_attribute("amount", config.amount.to_string())
        .add_attribute("valid_eth_sig", valid_eth_signature.verifies.to_string())
        .add_attribute("is_eligible", is_eligible.eligible.to_string())
        .add_attribute("minter_page", &config.minter_page))
}

fn remove_eth_address_from_eligible(deps: DepsMut, eth_address: String) {
    let address_exists = ELIGIBLE_ETH_ADDRS.load(deps.storage, &eth_address);
    match address_exists {
        Ok(_) => ELIGIBLE_ETH_ADDRS.save(deps.storage, &eth_address, &false).unwrap(), 
        Err(_) => ()
    }
}

fn airdrop_check_eligible(deps: Deps, eth_address: String) -> StdResult<EligibleResponse> {
    let is_eligible_addr =
        ELIGIBLE_ETH_ADDRS.load(deps.storage, &eth_address);
    match is_eligible_addr {
        Ok(is_eligible) => Ok(EligibleResponse {
            eligible: is_eligible,
        }),
        Err(_) => Ok(EligibleResponse { eligible: false }),
    }
}

fn add_eligible_eth(
    deps: DepsMut,
    eth_address: String,
    sender: Addr,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if sender != config.admin {
        return Err(ContractError::Unauthorized { sender });
    }
    let _ = ELIGIBLE_ETH_ADDRS.save(deps.storage, &eth_address, &true);
    Ok(Response::new())
}

#[allow(dead_code)]
#[allow(unused_variables)]
fn airdrop_check_valid(deps: Deps, env: Env, msg: QueryMsg) -> bool {
    true
}
