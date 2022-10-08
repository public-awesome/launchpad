use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, ExecuteMsg, HasEndedResponse, HasMemberResponse,
    HasStartedResponse, InstantiateMsg, IsActiveResponse, QueryMsg, MerkleRootResponse,
};
use crate::state::{Config, CONFIG, MERKLE_ROOT};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, StdResult};
use cosmwasm_std::{Timestamp};
use cw2::set_contract_version;
use cw_utils::{must_pay};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use sg1::checked_fair_burn;
use sg_std::{Response, GENESIS_MINT_START_TIME, NATIVE_DENOM};
use sha2::Digest;
use std::convert::TryInto;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg-whitelist-merkle";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// contract governance params
const PRICE_PER_1000_MEMBERS: u128 = 100_000_000;
const MIN_MINT_PRICE: u128 = 25_000_000;
const MAX_PER_ADDRESS_LIMIT: u32 = 30;


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mut msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    if msg.mint_price.denom != NATIVE_DENOM {
        return Err(ContractError::InvalidDenom(msg.mint_price.denom));
    }

    if msg.mint_price.amount.u128() < MIN_MINT_PRICE {
        return Err(ContractError::InvalidUnitPrice(
            msg.mint_price.amount.u128(),
            MIN_MINT_PRICE,
        ));
    }

    // Check per address limit is valid
    if msg.per_address_limit > MAX_PER_ADDRESS_LIMIT {
        return Err(ContractError::InvalidPerAddressLimit {
            max: MAX_PER_ADDRESS_LIMIT.to_string(),
            got: msg.per_address_limit.to_string(),
        });
    }
    if msg.per_address_limit == 0 {
        return Err(ContractError::InvalidPerAddressLimit {
            max: "must be > 0".to_string(),
            got: msg.per_address_limit.to_string(),
        });
    }

    let creation_fee = Decimal::new(msg.member_limit.into(), 3)
        .ceil()
        .to_u128()
        .unwrap()
        * PRICE_PER_1000_MEMBERS;
    let payment = must_pay(&info, NATIVE_DENOM)?;
    if payment.u128() != creation_fee {
        return Err(ContractError::IncorrectCreationFee(
            payment.u128(),
            creation_fee,
        ));
    }

    let config = Config {
        admin: info.sender.clone(),
        start_time: msg.start_time,
        end_time: msg.end_time,
        mint_price: msg.mint_price,
        merkle_root: msg.merkle_root.clone(),
    };
    CONFIG.save(deps.storage, &config)?;

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
    checked_fair_burn(&info, creation_fee, None, &mut res)?;

    MERKLE_ROOT.save(deps.storage, &msg.merkle_root);

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
        ExecuteMsg::UpdateStartTime(time) => execute_update_start_time(deps, env, info, time),
        ExecuteMsg::UpdateEndTime(time) => execute_update_end_time(deps, env, info, time),
    }
}

pub fn execute_update_start_time(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    start_time: Timestamp,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

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
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    // don't allow updating end time if whitelist is active
    if env.block.time >= config.start_time {
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
        QueryMsg::MerkleRoot {} => {
            to_binary(&query_merkle_root(deps)?)
        }

        QueryMsg::HasStarted {} => to_binary(&query_has_started(deps, env)?),
        QueryMsg::HasEnded {} => to_binary(&query_has_ended(deps, env)?),
        QueryMsg::IsActive {} => to_binary(&query_is_active(deps, env)?),
        QueryMsg::HasMember { member, proof } => to_binary(&query_has_member(deps, member, proof)?),
        QueryMsg::Config {} => to_binary(&query_config(deps, env)?),
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

fn query_merkle_root(
    deps: Deps,
) -> StdResult<MerkleRootResponse> {
    let result = MERKLE_ROOT.load(deps.storage)?;
    Ok(MerkleRootResponse { merkle_root: result })
}

fn query_has_member(deps: Deps, member: String, proof: Vec<String>) -> StdResult<HasMemberResponse> {
    let merkle_root = MERKLE_ROOT.load(deps.storage).unwrap_or_default();
    // println!("merkle_root {}", merkle_root);
    println!("member {}", member);

    let hash = sha2::Sha256::digest(member.as_bytes())
        .as_slice()
        .try_into()
        .map_err(|_| ContractError::Unauthorized {}).unwrap_or_default();
    println!("hash {:?}", hash);


    let hash = proof.into_iter().try_fold(hash, |hash, p| {
        let mut proof_buf = [0; 32];
        hex::decode_to_slice(p, &mut proof_buf).unwrap_or_default();
        let hashes = [hash, proof_buf];
        sha2::Sha256::digest(&hashes.concat())
            .as_slice()
            .try_into()
            .map_err(|_| ContractError::Unauthorized {})
    }).unwrap_or_default();
    println!("hash {:?}", hash);


    let mut root_buf: [u8; 32] = [0; 32];
    hex::decode_to_slice(merkle_root , &mut root_buf).unwrap_or_default();

    println!("hash {:?}", hash);
    println!("root_buf {:?}", root_buf);

    if root_buf != hash {
        return Ok(HasMemberResponse{
            has_member: false,
        });
    }

    Ok(HasMemberResponse {
        has_member: true,
    })
}

fn query_config(deps: Deps, env: Env) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        merkle_root: config.merkle_root,
        start_time: config.start_time,
        end_time: config.end_time,
        mint_price: config.mint_price,
        is_active: (env.block.time >= config.start_time) && (env.block.time < config.end_time),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{
        coin,
        testing::{mock_dependencies, mock_env, mock_info},
    };
    use sg_std::NATIVE_DENOM;

    const ADMIN: &str = "admin";
    const UNIT_AMOUNT: u128 = 100_000_000;

    const GENESIS_START_TIME: Timestamp = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    const END_TIME: Timestamp = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 1000);

    fn setup_contract(deps: DepsMut) {
        let msg = InstantiateMsg {
            merkle_root: "b097c35bb87dc749db869ee8f28b49ac53f6ab817a9e1bab5f1eec98223d416f".to_string(),
            start_time: GENESIS_START_TIME,
            end_time: END_TIME,
            mint_price: coin(UNIT_AMOUNT, NATIVE_DENOM),
            per_address_limit: 1,
            member_limit: 1000,
        };
        let info = mock_info(ADMIN, &[coin(100_000_000, "ustars")]);
        let res = instantiate(deps, mock_env(), info, msg).unwrap();
        assert_eq!(2, res.messages.len());
    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
    }

    #[test]
    fn improper_initialization() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            merkle_root: "0x7075152d03a5cd92104887b476862778ec0c87be5c2fa1c0a90f87c49fad6eff".to_string(),
            start_time: END_TIME,
            end_time: END_TIME,
            mint_price: coin(1, NATIVE_DENOM),
            per_address_limit: 1,
            member_limit: 1000,
        };
        let info = mock_info(ADMIN, &[coin(100_000_000, "ustars")]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    }

    #[test]
    fn improper_initialization_invalid_denom() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            merkle_root: "0x7075152d03a5cd92104887b476862778ec0c87be5c2fa1c0a90f87c49fad6eff".to_string(),
            start_time: END_TIME,
            end_time: END_TIME,
            mint_price: coin(UNIT_AMOUNT, "not_ustars"),
            per_address_limit: 1,
            member_limit: 1000,
        };
        let info = mock_info(ADMIN, &[coin(100_000_000, "ustars")]);
        let err = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(err.to_string(), "InvalidDenom: not_ustars");
    }

    #[test]
    fn improper_initialization_invalid_creation_fee() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            merkle_root: "0x7075152d03a5cd92104887b476862778ec0c87be5c2fa1c0a90f87c49fad6eff".to_string(),
            start_time: END_TIME,
            end_time: END_TIME,
            mint_price: coin(UNIT_AMOUNT, "ustars"),
            per_address_limit: 1,
            member_limit: 3000,
        };
        let info = mock_info(ADMIN, &[coin(100_000_000, "ustars")]);
        let err = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(
            err.to_string(),
            "IncorrectCreationFee 100000000 < 300000000"
        );
    }

    #[test]
    fn check_start_time_after_end_time() {
        let msg = InstantiateMsg {
            merkle_root: "0x7075152d03a5cd92104887b476862778ec0c87be5c2fa1c0a90f87c49fad6eff".to_string(),
            start_time: END_TIME,
            end_time: GENESIS_START_TIME,
            mint_price: coin(UNIT_AMOUNT, NATIVE_DENOM),
            per_address_limit: 1,
            member_limit: 1000,
        };
        let info = mock_info(ADMIN, &[coin(100_000_000, "ustars")]);
        let mut deps = mock_dependencies();
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    }

    #[test]
    fn update_start_time() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());

        let msg = ExecuteMsg::UpdateStartTime(Timestamp::from_nanos(GENESIS_MINT_START_TIME - 100));
        let info = mock_info(ADMIN, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.attributes.len(), 3);
        let res = query_config(deps.as_ref(), mock_env()).unwrap();
        assert_eq!(res.start_time, GENESIS_START_TIME);
    }

    #[test]
    fn update_end_time() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());

        let msg = ExecuteMsg::UpdateEndTime(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 100));
        let info = mock_info(ADMIN, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.attributes.len(), 3);

        let msg = ExecuteMsg::UpdateEndTime(Timestamp::from_nanos(GENESIS_MINT_START_TIME - 100));
        let info = mock_info(ADMIN, &[]);
        execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    }

    #[test]
    fn query_membership() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());

        let valid_user = mock_info("stars1ye63jpm474yfrq02nyplrspyw75y82tptsls9t", &[]);
        let proof = vec![
            "8b231ee54c7e265bca6482e6a6a0f251c5da97be74f4e9720ae81a1bc08beea9".to_string(), 
            "28fc41471ab92238e98664e99671e906cb29c048dd0343f3acf5295e424270e1".to_string(),
        ];
        let res = query_has_member(deps.as_ref(), valid_user.sender.to_string(), proof).unwrap();
        assert_eq!(res.has_member, true);

        // let valid_user = mock_info("b", &[]);
        // let proof = "".to_string();
        // let res = query_has_member(deps.as_ref(), valid_user.sender.to_string(), proof).unwrap();
        // assert_eq!(res.has_member, true);

        // let valid_user = mock_info("c", &[]);
        // let proof = "".to_string();
        // let res = query_has_member(deps.as_ref(), valid_user.sender.to_string(), proof).unwrap();
        // assert_eq!(res.has_member, true);

        // let valid_user = mock_info("d", &[]);
        // let proof = "".to_string();
        // let res = query_has_member(deps.as_ref(), valid_user.sender.to_string(), proof).unwrap();
        // assert_eq!(res.has_member, false);
    }
}
