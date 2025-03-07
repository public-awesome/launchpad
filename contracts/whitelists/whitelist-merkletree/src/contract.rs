use crate::admin::{
    can_execute, execute_freeze, execute_update_admins, query_admin_list, query_can_execute,
};
use crate::error::ContractError;
use crate::helpers::crypto::{string_to_byte_slice, valid_hash_string, verify_merkle_root};
use crate::helpers::utils::verify_tree_uri;
use crate::helpers::validators::map_validate;
use crate::msg::{
    ConfigResponse, ExecuteMsg, HasEndedResponse, HasMemberResponse, HasStartedResponse,
    InstantiateMsg, IsActiveResponse, MerkleRootResponse, MerkleTreeURIResponse, QueryMsg,
};
use crate::state::{AdminList, Config, ADMIN_LIST, CONFIG, MERKLE_ROOT, MERKLE_TREE_URI};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Empty, Env, Event, MessageInfo, Response, StdError,
    StdResult, Timestamp,
};
use cw2::set_contract_version;
use cw_utils::must_pay;
use sg_utils::{GENESIS_MINT_START_TIME, NATIVE_DENOM};

use rs_merkle::{algorithms::Sha256, Hasher};
use semver::Version;
use sg1::checked_fair_burn;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:whitelist-merkletree";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// contract governance params
pub const CREATION_FEE: u128 = 1_000_000_000;
pub const MIN_MINT_PRICE: u128 = 0;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    verify_merkle_root(&msg.merkle_root)?;
    verify_tree_uri(&msg.merkle_tree_uri)?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let payment = must_pay(&info, NATIVE_DENOM)?;
    if payment.u128() != CREATION_FEE {
        return Err(ContractError::IncorrectCreationFee(
            payment.u128(),
            CREATION_FEE,
        ));
    }

    if msg.start_time > msg.end_time {
        return Err(ContractError::InvalidStartTime(
            msg.start_time,
            msg.end_time,
        ));
    }

    if env.block.time >= msg.start_time {
        return Err(ContractError::InvalidStartTime(
            env.block.time,
            msg.start_time,
        ));
    }

    let genesis_start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    if msg.start_time < genesis_start_time {
        return Err(ContractError::InvalidStartTime(
            msg.start_time,
            genesis_start_time,
        ));
    }

    let mut res = Response::new();
    checked_fair_burn(&info, &env, CREATION_FEE, None, &mut res)?;

    let config = Config {
        start_time: msg.start_time,
        end_time: msg.end_time,
        mint_price: msg.mint_price,
        per_address_limit: msg.per_address_limit,
    };

    let admin_config = AdminList {
        admins: map_validate(deps.api, &msg.admins)?,
        mutable: msg.admins_mutable,
    };

    MERKLE_ROOT.save(deps.storage, &msg.merkle_root)?;
    ADMIN_LIST.save(deps.storage, &admin_config)?;
    CONFIG.save(deps.storage, &config)?;

    let tree_url = msg.merkle_tree_uri.unwrap_or_default();

    let mut attrs = Vec::with_capacity(6);

    attrs.push(("action", "update_merkle_tree"));
    attrs.push(("merkle_root", &msg.merkle_root));
    attrs.push(("contract_name", CONTRACT_NAME));
    attrs.push(("contract_version", CONTRACT_VERSION));
    if !tree_url.is_empty() {
        attrs.push(("merkle_tree_uri", &tree_url));
    }
    attrs.push(("sender", info.sender.as_str()));

    Ok(res.add_attributes(attrs))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateStartTime(time) => execute_update_start_time(deps, env, info, time),
        ExecuteMsg::UpdateEndTime(time) => execute_update_end_time(deps, env, info, time),
        ExecuteMsg::UpdateAdmins { admins } => execute_update_admins(deps, env, info, admins),
        ExecuteMsg::Freeze {} => execute_freeze(deps, env, info),
    }
}

pub fn execute_update_merkle_tree(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    merkle_root: String,
    merkle_tree_uri: Option<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    can_execute(&deps, info.sender.clone())?;
    verify_merkle_root(&merkle_root)?;
    verify_tree_uri(&merkle_tree_uri)?;

    if env.block.time < config.end_time {
        return Err(ContractError::AlreadyEnded {});
    }

    MERKLE_ROOT.save(deps.storage, &merkle_root)?;

    let mut attrs = Vec::with_capacity(4);

    attrs.push(("action", String::from("update_merkle_tree")));
    attrs.push(("merkle_root", merkle_root));
    if let Some(uri) = merkle_tree_uri {
        attrs.push(("merkle_tree_uri", uri));
    }
    attrs.push(("sender", info.sender.to_string()));

    Ok(Response::new().add_attributes(attrs))
}

pub fn execute_update_start_time(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    start_time: Timestamp,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    can_execute(&deps, info.sender.clone())?;

    // don't allow updating start time if whitelist is active
    if env.block.time >= config.start_time {
        return Err(ContractError::AlreadyStarted {});
    }

    if start_time > config.end_time {
        return Err(ContractError::InvalidStartTime(start_time, config.end_time));
    }

    let genesis_start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    let start_time = if start_time < genesis_start_time {
        genesis_start_time
    } else {
        start_time
    };

    config.start_time = start_time;
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "update_start_time")
        .add_attribute("start_time", start_time.to_string())
        .add_attribute("sender", info.sender))
}

pub fn execute_update_end_time(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    end_time: Timestamp,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    can_execute(&deps, info.sender.clone())?;

    if env.block.time >= config.start_time && end_time > config.end_time {
        return Err(ContractError::AlreadyStarted {});
    }

    if end_time < config.start_time {
        return Err(ContractError::InvalidEndTime(end_time, config.start_time));
    }

    config.end_time = end_time;
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "update_end_time")
        .add_attribute("end_time", end_time.to_string())
        .add_attribute("sender", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::HasStarted {} => to_json_binary(&query_has_started(deps, env)?),
        QueryMsg::HasEnded {} => to_json_binary(&query_has_ended(deps, env)?),
        QueryMsg::IsActive {} => to_json_binary(&query_is_active(deps, env)?),
        QueryMsg::HasMember {
            member,
            proof_hashes,
        } => to_json_binary(&query_has_member(deps, member, proof_hashes)?),
        QueryMsg::Config {} => to_json_binary(&query_config(deps, env)?),
        QueryMsg::AdminList {} => to_json_binary(&query_admin_list(deps)?),
        QueryMsg::CanExecute { sender, .. } => to_json_binary(&query_can_execute(deps, &sender)?),
        QueryMsg::MerkleRoot {} => to_json_binary(&query_merkle_root(deps)?),
        QueryMsg::MerkleTreeURI {} => to_json_binary(&query_merkle_tree_uri(deps)?),
    }
}

fn query_has_started(deps: Deps, env: Env) -> StdResult<HasStartedResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(HasStartedResponse {
        has_started: (env.block.time >= config.start_time),
    })
}

fn query_has_ended(deps: Deps, env: Env) -> StdResult<HasEndedResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(HasEndedResponse {
        has_ended: (env.block.time >= config.end_time),
    })
}

fn query_is_active(deps: Deps, env: Env) -> StdResult<IsActiveResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(IsActiveResponse {
        is_active: (env.block.time >= config.start_time) && (env.block.time < config.end_time),
    })
}

pub fn query_has_member(
    deps: Deps,
    member: String,
    proof_hashes: Vec<String>,
) -> StdResult<HasMemberResponse> {
    let merkle_root = MERKLE_ROOT.load(deps.storage)?;

    let member_init_hash_slice = Sha256::hash(member.as_bytes());

    let final_hash = proof_hashes.into_iter().try_fold(
        member_init_hash_slice,
        |accum_hash_slice, new_proof_hashstring| {
            valid_hash_string(&new_proof_hashstring)?;

            let mut hashe_slices = [
                accum_hash_slice,
                string_to_byte_slice(&new_proof_hashstring)?,
            ];
            hashe_slices.sort_unstable();
            Result::<[u8; 32], StdError>::Ok(Sha256::hash(&hashe_slices.concat()))
        },
    );

    if final_hash.is_err() {
        return Err(cosmwasm_std::StdError::GenericErr {
            msg: "Invalid Merkle Proof".to_string(),
        });
    }

    Ok(HasMemberResponse {
        has_member: merkle_root == hex::encode(final_hash.unwrap()),
    })
}

pub fn query_config(deps: Deps, env: Env) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        num_members: 0,
        member_limit: 0,
        per_address_limit: config.per_address_limit,
        start_time: config.start_time,
        end_time: config.end_time,
        mint_price: config.mint_price,
        is_active: (env.block.time >= config.start_time) && (env.block.time < config.end_time),
    })
}

pub fn query_merkle_root(deps: Deps) -> StdResult<MerkleRootResponse> {
    Ok(MerkleRootResponse {
        merkle_root: MERKLE_ROOT.load(deps.storage)?,
    })
}

pub fn query_merkle_tree_uri(deps: Deps) -> StdResult<MerkleTreeURIResponse> {
    Ok(MerkleTreeURIResponse {
        merkle_tree_uri: MERKLE_TREE_URI.may_load(deps.storage)?,
    })
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
