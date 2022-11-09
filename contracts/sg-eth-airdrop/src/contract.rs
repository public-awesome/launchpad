#[cfg(not(feature = "library"))]
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG};

use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, StdResult};
use cw2::set_contract_version;
use cw_utils::parse_reply_instantiate_data;
use sg_std::Response;
use whitelist_generic::helpers::WhitelistGenericContract;

use crate::build_msg::{build_bank_message, build_whitelist_instantiate_msg};
use crate::computation::compute_valid_eth_sig;
use crate::constants::{CONTRACT_NAME, CONTRACT_VERSION, INIT_WHITELIST_REPLY_ID};
use crate::responses::{get_add_eligible_eth_response, get_remove_eligible_eth_response};

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
        claim_msg_plaintext: msg.clone().claim_msg_plaintext,
        airdrop_amount: msg.airdrop_amount,
        minter_page: msg.clone().minter_page,
        whitelist_address: None,
    };
    CONFIG.save(deps.storage, &cfg)?;

    let whitelist_instantiate_msg = build_whitelist_instantiate_msg(env, msg);

    let res = Response::new();
    Ok(res
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION)
        .add_attribute("sender", info.sender)
        .add_submessage(whitelist_instantiate_msg))
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
        } => claim_airdrop(deps, info, eth_address, eth_sig),
        ExecuteMsg::AddEligibleEth { eth_addresses } => {
            get_add_eligible_eth_response(deps, info, eth_addresses)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::AirdropEligible { eth_address } => {
            to_binary(&airdrop_check_eligible(deps, eth_address)?)
        }
    }
}

fn claim_airdrop(
    deps: DepsMut,
    info: MessageInfo,
    eth_address: String,
    eth_sig: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let is_eligible = airdrop_check_eligible(deps.as_ref(), eth_address.clone()).unwrap();
    let valid_eth_signature =
        compute_valid_eth_sig(&deps, info.clone(), &config, eth_sig, eth_address.clone());

    let (mut res, mut claimed_amount) = (Response::new(), 0);
    if is_eligible && valid_eth_signature.verifies {
        res = get_remove_eligible_eth_response(deps, eth_address).unwrap();
        res = res.add_submessage(build_bank_message(info, config.airdrop_amount));
        claimed_amount = config.airdrop_amount;
    }
    Ok(res
        .add_attribute("claimed_amount", claimed_amount.to_string())
        .add_attribute("valid_eth_sig", valid_eth_signature.verifies.to_string())
        .add_attribute("eligible_at_request", is_eligible.to_string())
        .add_attribute("minter_page", &config.minter_page))
}

fn airdrop_check_eligible(deps: Deps, eth_address: String) -> StdResult<bool> {
    let config = CONFIG.load(deps.storage)?;
    match config.whitelist_address {
        Some(address) => WhitelistGenericContract(deps.api.addr_validate(&address)?)
            .includes(&deps.querier, eth_address),
        None => Err(cosmwasm_std::StdError::NotFound {
            kind: "Whitelist Contract".to_string(),
        }),
    }
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
