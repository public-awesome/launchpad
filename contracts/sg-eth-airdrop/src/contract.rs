#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{EligibleResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::signature_verify::{query_verify_cosmos, query_verify_ethereum_text};
use crate::state::{Config, CONFIG, ELIGIBLE_ETH_ADDRS};
use sg_std::Response;

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
        admin: info.sender.clone(),
        claim_msg_plaintext: msg.config.claim_msg_plaintext,
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
        ExecuteMsg::AddEligibleEth { eth_address } => {
            add_eligible_eth(deps, eth_address, info.sender)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
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
    stargaze_address: String,
    stargaze_sig: String,
) -> Result<Response, ContractError> {
    let amount: u32 = 30000;
    let minter_page: String = "http://levana_page/airdrop".to_string();
    let eth_sig_hex = hex::decode(eth_sig).unwrap();
    // let sg_sig_hex = hex::decode(stargaze_sig).unwrap();
    let config = CONFIG.load(deps.storage)?;

    let valid_eth_signature = query_verify_ethereum_text(
        deps.as_ref(),
        &config.claim_msg_plaintext,
        &eth_sig_hex,
        &eth_address,
    )
    .unwrap();
    // let valid_cosmos_sig = query_verify_cosmos(
    //     deps.as_ref(),
    //     config.claim_msg_plaintext.as_bytes(),
    //     &stargaze_sig.as_bytes(),
    //     stargaze_address.as_bytes(),
    // )
    // .unwrap();
    let valid_cosmos_sig = false; 

    let valid_claim: bool = valid_eth_signature.verifies && valid_cosmos_sig; 
    Ok(Response::new()
        .add_attribute("amount", amount.to_string())
        .add_attribute("valid_eth_sig", valid_eth_signature.verifies.to_string())
        .add_attribute("valid_cosmos_sig", valid_cosmos_sig.to_string())
        .add_attribute("valid_claim", valid_claim.to_string())
        .add_attribute("minter_page", minter_page))
}

fn airdrop_check_eligible(deps: Deps, eth_address: Addr) -> StdResult<EligibleResponse> {
    let is_eligible_addr =
        ELIGIBLE_ETH_ADDRS.load(deps.storage, &Addr::unchecked(eth_address.to_string()));
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
        return Err(ContractError::Unauthorized { sender: sender });
    }
    let result = ELIGIBLE_ETH_ADDRS.save(
        deps.storage,
        &Addr::unchecked(eth_address.to_string()),
        &true,
    );
    Ok(Response::new())
}

// internal function only
fn airdrop_check_valid(deps: Deps, env: Env, msg: QueryMsg) -> bool {
    true
}
