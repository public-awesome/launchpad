#[cfg(not(feature = "library"))]
use crate::error::ContractError;
use crate::msg::{EligibleResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
#[allow(unused_imports)]
use crate::signature_verify::{query_verify_cosmos, query_verify_ethereum_text};
use crate::state::{Config, CONFIG, ELIGIBLE_ETH_ADDRS};

use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, StdResult, WasmMsg,
};
use cw2::set_contract_version;
use cw_utils::parse_reply_instantiate_data;
use sg_std::{Response, SubMsg};
use whitelist_generic::helpers::WhitelistUpdatableContract;
use whitelist_generic::msg::ExecuteMsg as WGExecuteMsg;
use whitelist_generic::msg::InstantiateMsg as WGInstantiateMsg;
const INIT_WHITELIST_REPLY_ID: u64 = 1;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg-eth-airdrop";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const GENERIC_WHITELIST_LABEL: &str = "Generic Whitelist for Airdrop";

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
        claim_msg_plaintext: msg.claim_msg_plaintext,
        amount: msg.amount,
        minter_page: msg.minter_page,
        whitelist_address: None,
    };
    CONFIG.save(deps.storage, &cfg)?;

    let whitelist_instantiate_msg = WGInstantiateMsg {
        addresses: msg.addresses,
        mint_discount_bps: Some(0),
        per_address_limit: 1,
    };
    let wasm_msg = WasmMsg::Instantiate {
        code_id: msg.minter_code_id,
        admin: Some(env.contract.address.to_string()),
        funds: info.funds,
        label: GENERIC_WHITELIST_LABEL.to_string(),
        msg: to_binary(&whitelist_instantiate_msg)?,
    };
    let submsg = SubMsg::reply_on_success(wasm_msg, INIT_WHITELIST_REPLY_ID);

    let res = Response::new();
    Ok(res
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION)
        .add_attribute("sender", info.sender)
        .add_submessage(submsg))
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
            eth_sig,
        } => claim_airdrop(deps, eth_address, eth_sig, info),
        ExecuteMsg::AddEligibleEth { eth_addresses } => add_eligible_eth(deps, eth_addresses, info),
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
    let is_eligible = EligibleResponse { eligible: true };
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
        Ok(_) => ELIGIBLE_ETH_ADDRS
            .save(deps.storage, &eth_address, &false)
            .unwrap(),
        Err(_) => (),
    }
}

fn airdrop_check_eligible(deps: Deps, eth_address: String) -> StdResult<bool> {
    let config = CONFIG.load(deps.storage)?;
    match config.whitelist_address {
        Some(address) => WhitelistUpdatableContract(deps.api.addr_validate(&address)?)
            .includes(&deps.querier, eth_address),
        None => Err(cosmwasm_std::StdError::NotFound {
            kind: "Whitelist Contract".to_string(),
        }),
    }
}

fn add_eligible_eth(
    deps: DepsMut,
    addresses: Vec<String>,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {
            sender: info.sender,
        });
    }
    let whitelist_address = match config.whitelist_address {
        Some(address) => address,
        None => return Err(ContractError::WhitelistContractNotSet2 {}),
    };
    let execute_msg = WGExecuteMsg::AddAddresses {
        addresses: addresses,
    };
    let mut res = Response::new();
    res = res.add_message(
        WhitelistUpdatableContract(deps.api.addr_validate(&whitelist_address)?)
            .call(execute_msg)?,
    );
    Ok(res)
}

#[allow(dead_code)]
#[allow(unused_variables)]
fn airdrop_check_valid(deps: Deps, env: Env, msg: QueryMsg) -> bool {
    true
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    if msg.id != INIT_WHITELIST_REPLY_ID {
        return Err(ContractError::InvalidReplyID {});
    }

    let reply = parse_reply_instantiate_data(msg);
    match reply {
        Ok(res) => {
            let whitelist_address = &res.contract_address;
            let mut config = CONFIG.load(deps.storage)?;
            config.whitelist_address = Some(whitelist_address.to_string());
            CONFIG.save(deps.storage, &config)?;

            Ok(Response::default()
                .add_attribute("action", "init_whitelist_reply")
                .add_attribute("whitelist_address", whitelist_address))
        }
        Err(_) => Err(ContractError::ReplyOnSuccess {}),
    }
}
