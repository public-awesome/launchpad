use crate::admin::{
    can_execute, execute_freeze, execute_update_admins, query_admin_list, query_can_execute,
};
use crate::error::ContractError;
use crate::helpers::crypto::{string_to_byte_slice, valid_hash_string, verify_merkle_root};
use crate::helpers::utils::{
    fetch_active_stage, fetch_active_stage_index, validate_stages, validate_update, verify_tree_uri,
};
use crate::helpers::validators::map_validate;
use crate::msg::{
    ConfigResponse, ExecuteMsg, HasEndedResponse, HasMemberResponse, HasStartedResponse,
    InstantiateMsg, IsActiveResponse, MerkleRootResponse, MerkleTreeURIResponse, QueryMsg,
    StageResponse, StagesResponse, UpdateStageConfigMsg,
};
use crate::state::{AdminList, Config, Stage, ADMIN_LIST, CONFIG, MERKLE_ROOTS, MERKLE_TREE_URIS};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure, to_json_binary, Binary, Coin, Deps, DepsMut, Empty, Env, Event, MessageInfo, Response,
    StdError, StdResult, Timestamp, Uint128,
};
use cw2::set_contract_version;
use cw_utils::must_pay;
use sg_std::NATIVE_DENOM;

use rs_merkle::{algorithms::Sha256, Hasher};
use semver::Version;
use sg1::checked_fair_burn;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:tiered-whitelist-merkletree";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// contract governance params
pub const CREATION_FEE: u128 = 1_000_000_000;
pub const MIN_MINT_PRICE: u128 = 0;
pub const MAX_PER_ADDRESS_LIMIT: u32 = 50;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    for merkle_root in msg.merkle_roots.iter() {
        verify_merkle_root(merkle_root)?;
    }
    if let Some(tree_uris) = msg.merkle_tree_uris.as_ref() {
        for uri in tree_uris.iter() {
            verify_tree_uri(uri)?;
        }
    }
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let payment = must_pay(&info, NATIVE_DENOM)?;
    if payment.u128() != CREATION_FEE {
        return Err(ContractError::IncorrectCreationFee(
            payment.u128(),
            CREATION_FEE,
        ));
    }

    validate_stages(&env, &msg.stages)?;

    let mut res = Response::new();
    checked_fair_burn(&info, CREATION_FEE, None, &mut res)?;

    let config = Config { stages: msg.stages };

    let admin_config = AdminList {
        admins: map_validate(deps.api, &msg.admins)?,
        mutable: msg.admins_mutable,
    };

    MERKLE_ROOTS.save(deps.storage, &msg.merkle_roots)?;
    ADMIN_LIST.save(deps.storage, &admin_config)?;
    CONFIG.save(deps.storage, &config)?;

    let tree_uris = msg.merkle_tree_uris.unwrap_or_default();
    if !tree_uris.is_empty() {
        MERKLE_TREE_URIS.save(deps.storage, &tree_uris.clone())?;
    }

    let mut attrs = Vec::with_capacity(6);

    attrs.push(("action", "update_merkle_tree"));
    let merkle_roots_joined = msg.merkle_roots.join(",");
    attrs.push(("merkle_roots", &merkle_roots_joined));
    attrs.push(("contract_name", CONTRACT_NAME));
    attrs.push(("contract_version", CONTRACT_VERSION));
    let tree_uris_joined = tree_uris.join(",");
    if !tree_uris.is_empty() {
        attrs.push(("merkle_tree_uris", &tree_uris_joined));
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
        ExecuteMsg::UpdateStageConfig(msg) => execute_update_stage_config(deps, env, info, msg),
        ExecuteMsg::UpdateAdmins { admins } => execute_update_admins(deps, env, info, admins),
        ExecuteMsg::Freeze {} => execute_freeze(deps, env, info),
    }
}

pub fn execute_update_merkle_tree(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    merkle_roots: Vec<String>,
    merkle_tree_uris: Option<Vec<String>>,
) -> Result<Response, ContractError> {
    can_execute(&deps, info.sender.clone())?;
    let config = CONFIG.load(deps.storage)?;

    for merkle_root in merkle_roots.iter() {
        verify_merkle_root(merkle_root)?;
    }

    if let Some(tree_uris) = merkle_tree_uris.as_ref() {
        for uri in tree_uris.iter() {
            verify_tree_uri(uri)?;
        }
    }

    ensure!(
        config
            .stages
            .iter()
            .all(|stage| stage.end_time <= env.block.time),
        ContractError::AlreadyEnded {}
    );

    MERKLE_ROOTS.save(deps.storage, &merkle_roots)?;

    let tree_uris = merkle_tree_uris.clone().unwrap_or_default();
    if !tree_uris.is_empty() {
        MERKLE_TREE_URIS.save(deps.storage, &tree_uris)?;
    }

    let mut attrs = Vec::with_capacity(4);

    attrs.push(("action", String::from("update_merkle_tree")));
    attrs.push(("merkle_roots", merkle_roots.join(",")));
    if let Some(uris) = merkle_tree_uris.clone() {
        attrs.push(("merkle_tree_uris", uris.join(",")));
    }
    attrs.push(("sender", info.sender.to_string()));

    Ok(Response::new().add_attributes(attrs))
}

pub fn execute_update_stage_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: UpdateStageConfigMsg,
) -> Result<Response, ContractError> {
    can_execute(&deps, info.sender.clone())?;
    let mut config = CONFIG.load(deps.storage)?;
    let stage_id = msg.stage_id as usize;
    let updated_stage = Stage {
        name: msg.name.unwrap_or(config.stages[stage_id].clone().name),
        start_time: msg
            .start_time
            .unwrap_or(config.stages[stage_id].clone().start_time),
        end_time: msg
            .end_time
            .unwrap_or(config.stages[stage_id].clone().end_time),
        mint_price: msg
            .mint_price
            .unwrap_or(config.stages[stage_id].clone().mint_price),
        per_address_limit: msg
            .per_address_limit
            .unwrap_or(config.stages[stage_id].clone().per_address_limit),
        mint_count_limit: msg
            .mint_count_limit
            .unwrap_or(config.stages[stage_id].clone().mint_count_limit),
    };
    config.stages[stage_id] = updated_stage.clone();
    validate_update(&env, &config.stages)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_stage_config")
        .add_attribute("stage_id", stage_id.to_string())
        .add_attribute("name", updated_stage.clone().name)
        .add_attribute("start_time", updated_stage.clone().start_time.to_string())
        .add_attribute("end_time", updated_stage.clone().end_time.to_string())
        .add_attribute("mint_price", updated_stage.clone().mint_price.to_string())
        .add_attribute(
            "per_address_limit",
            updated_stage.clone().per_address_limit.to_string(),
        )
        .add_attribute("sender", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::HasStarted {} => to_json_binary(&query_has_started(deps, env)?),
        QueryMsg::HasEnded {} => to_json_binary(&query_has_ended(deps, env)?),
        QueryMsg::IsActive {} => to_json_binary(&query_is_active(deps, env)?),
        QueryMsg::ActiveStage {} => to_json_binary(&fetch_active_stage(deps.storage, &env)),
        QueryMsg::ActiveStageId {} => {
            to_json_binary(&fetch_active_stage_index(deps.storage, &env).map_or(0, |i| i + 1))
        }
        QueryMsg::HasMember {
            member,
            proof_hashes,
        } => to_json_binary(&query_has_member(deps, member, env, proof_hashes)?),
        QueryMsg::Config {} => to_json_binary(&query_config(deps, env)?),
        QueryMsg::AdminList {} => to_json_binary(&query_admin_list(deps)?),
        QueryMsg::CanExecute { sender, .. } => to_json_binary(&query_can_execute(deps, &sender)?),
        QueryMsg::Stage { stage_id } => to_json_binary(&query_stage(deps, stage_id)?),
        QueryMsg::Stages {} => to_json_binary(&query_stage_list(deps)?),
        QueryMsg::MerkleRoots {} => to_json_binary(&query_merkle_roots(deps)?),
        QueryMsg::MerkleTreeURIs {} => to_json_binary(&query_merkle_tree_uris(deps)?),
    }
}

fn query_has_started(deps: Deps, env: Env) -> StdResult<HasStartedResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(HasStartedResponse {
        has_started: !config.stages.is_empty() && (env.block.time >= config.stages[0].start_time),
    })
}

fn query_has_ended(deps: Deps, env: Env) -> StdResult<HasEndedResponse> {
    let config = CONFIG.load(deps.storage)?;
    let stage_count = config.stages.len();
    Ok(HasEndedResponse {
        has_ended: (stage_count > 0) && (env.block.time >= config.stages[stage_count - 1].end_time),
    })
}

fn query_is_active(deps: Deps, env: Env) -> StdResult<IsActiveResponse> {
    Ok(IsActiveResponse {
        is_active: fetch_active_stage(deps.storage, &env).is_some(),
    })
}

pub fn query_has_member(
    deps: Deps,
    member: String,
    env: Env,
    proof_hashes: Vec<String>,
) -> StdResult<HasMemberResponse> {
    let active_stage = fetch_active_stage_index(deps.storage, &env)
        .ok_or_else(|| StdError::generic_err("No active stage found"))?;

    let merkle_root = MERKLE_ROOTS.load(deps.storage)?[active_stage as usize].clone();

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
    let active_stage = fetch_active_stage(deps.storage, &env);
    if let Some(stage) = active_stage {
        Ok(ConfigResponse {
            num_members: 0,
            per_address_limit: stage.per_address_limit,
            member_limit: 0,
            start_time: stage.start_time,
            end_time: stage.end_time,
            mint_price: stage.mint_price,
            is_active: true,
        })
    } else if !config.stages.is_empty() {
        let stage = if env.block.time < config.stages[0].start_time {
            config.stages[0].clone()
        } else {
            config.stages[config.stages.len() - 1].clone()
        };
        Ok(ConfigResponse {
            num_members: 0,
            per_address_limit: stage.per_address_limit,
            member_limit: 0,
            start_time: stage.start_time,
            end_time: stage.end_time,
            mint_price: stage.mint_price,
            is_active: false,
        })
    } else {
        Ok(ConfigResponse {
            num_members: 0,
            per_address_limit: 0,
            member_limit: 0,
            start_time: Timestamp::from_seconds(0),
            end_time: Timestamp::from_seconds(0),
            mint_price: Coin {
                denom: NATIVE_DENOM.to_string(),
                amount: Uint128::zero(),
            },
            is_active: false,
        })
    }
}

pub fn query_merkle_roots(deps: Deps) -> StdResult<MerkleRootResponse> {
    Ok(MerkleRootResponse {
        merkle_roots: MERKLE_ROOTS.load(deps.storage)?,
    })
}

pub fn query_merkle_tree_uris(deps: Deps) -> StdResult<MerkleTreeURIResponse> {
    Ok(MerkleTreeURIResponse {
        merkle_tree_uris: MERKLE_TREE_URIS.may_load(deps.storage)?,
    })
}

pub fn query_stage(deps: Deps, stage_id: u32) -> StdResult<StageResponse> {
    let config = CONFIG.load(deps.storage)?;
    ensure!(
        stage_id < config.stages.len() as u32,
        StdError::generic_err("Stage not found")
    );
    let merkle_root = MERKLE_ROOTS.load(deps.storage)?[stage_id as usize].clone();
    Ok(StageResponse {
        stage_id,
        stage: config.stages[stage_id as usize].clone(),
        merkle_root,
    })
}

pub fn query_stage_list(deps: Deps) -> StdResult<StagesResponse> {
    let config = CONFIG.load(deps.storage)?;
    ensure!(
        !config.stages.is_empty(),
        StdError::generic_err("No stages found")
    );
    let merkle_roots = MERKLE_ROOTS.load(deps.storage)?;
    let stages = config
        .stages
        .iter()
        .enumerate()
        .map(|(i, stage)| StageResponse {
            stage_id: i as u32,
            stage: stage.clone(),
            merkle_root: merkle_roots[i].clone(),
        })
        .collect();
    Ok(StagesResponse { stages })
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
