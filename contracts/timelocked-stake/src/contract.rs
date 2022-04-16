#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, DistributionMsg, Env, MessageInfo,
    Order, Response, StakingMsg, StdResult, Uint128,
};
use cw2::set_contract_version;
use cw_utils::{must_pay, nonpayable};
use sg_std::NATIVE_DENOM;

use crate::error::ContractError;
use crate::msg::{Delegation, DelegationsResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Stake, STAKE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:native-staking";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let stake = Stake {
        owner: info.sender,
        validator: deps.api.addr_validate(&msg.validator)?,
        end_time: env.block.time.plus_seconds(msg.min_duration),
        amount: must_pay(&info, NATIVE_DENOM)?,
    };
    STAKE.save(deps.storage, &stake)?;

    Ok(Response::new()
        .add_message(StakingMsg::Delegate {
            validator: msg.validator,
            amount: coin(stake.amount.u128(), NATIVE_DENOM),
        })
        .add_attribute("action", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let api = deps.api;

    match msg {
        ExecuteMsg::Undelegate {} => execute_undelegate(deps, env, info),
        ExecuteMsg::Redelegate { dst_validator } => {
            execute_redelegate(deps, info, api.addr_validate(&dst_validator)?)
        }
        ExecuteMsg::Claim {} => execute_claim(deps, info),
    }
}

pub fn execute_undelegate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let stake = STAKE.load(deps.storage)?;

    if env.block.time < stake.end_time {
        return Err(ContractError::StakeNotExpired {});
    }

    // TODO: what if this delegation has been slashed, would this undelegation amount still work?

    Ok(Response::default()
        .add_message(StakingMsg::Undelegate {
            validator: stake.validator.to_string(),
            amount: coin(stake.amount.u128(), NATIVE_DENOM),
        })
        .add_attribute("action", "undelegate"))
}

/// Redelegate to a new validator. Also updates the validator in storage.
pub fn execute_redelegate(
    deps: DepsMut,
    info: MessageInfo,
    dst_validator: Addr,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let mut stake = STAKE.load(deps.storage)?;

    let redelegate_msg = CosmosMsg::Staking(StakingMsg::Redelegate {
        src_validator: stake.validator.to_string(),
        dst_validator: dst_validator.to_string(),
        amount: coin(stake.amount.u128(), NATIVE_DENOM),
    });

    stake.validator = dst_validator;
    STAKE.save(deps.storage, &stake)?;

    Ok(Response::default()
        .add_attribute("action", "redelegate")
        .add_attribute("dst_validator", dst_validator)
        .add_message(redelegate_msg))
}

pub fn execute_claim(
    _deps: DepsMut,
    info: MessageInfo,
    validator: Addr,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let withdraw_reward_msg = CosmosMsg::Distribution(DistributionMsg::WithdrawDelegatorReward {
        validator: validator.to_string(),
    });

    Ok(Response::default()
        .add_attribute("action", "claim")
        .add_attribute("validator", validator)
        .add_message(withdraw_reward_msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let api = deps.api;

    match msg {
        QueryMsg::Delegations { address } => {
            to_binary(&query_delegated(deps, api.addr_validate(&address)?)?)
        }
    }
}

fn query_delegated(deps: Deps, address: Addr) -> StdResult<DelegationsResponse> {
    let delegations = STAKE
        .prefix(&address)
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            item.map(|(validator, stake)| Delegation {
                validator,
                stake: stake.amount,
                end_time: stake.end_time,
            })
        })
        .collect::<StdResult<Vec<_>>>()?;

    Ok(DelegationsResponse { delegations })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::coins;
    use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // // it worked, let's query the state
        // let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        // let value: CountResponse = from_binary(&res).unwrap();
        // assert_eq!(17, value.count);
    }

    #[test]
    fn increment() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // // beneficiary can release it
        // let info = mock_info("anyone", &coins(2, "token"));
        // let msg = ExecuteMsg::Increment {};
        // let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // // should increase counter by 1
        // let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        // let value: CountResponse = from_binary(&res).unwrap();
        // assert_eq!(18, value.count);
    }

    #[test]
    fn reset() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // // beneficiary can release it
        // let unauth_info = mock_info("anyone", &coins(2, "token"));
        // let msg = ExecuteMsg::Reset { count: 5 };
        // let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
        // match res {
        //     Err(ContractError::Unauthorized {}) => {}
        //     _ => panic!("Must return unauthorized error"),
        // }

        // // only the original creator can reset the counter
        // let auth_info = mock_info("creator", &coins(2, "token"));
        // let msg = ExecuteMsg::Reset { count: 5 };
        // let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        // // should now be 5
        // let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        // let value: CountResponse = from_binary(&res).unwrap();
        // assert_eq!(5, value.count);
    }
}
