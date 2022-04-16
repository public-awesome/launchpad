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
const CONTRACT_NAME: &str = "crates.io:staking";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
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
        ExecuteMsg::Delegate {
            validator,
            min_duration,
        } => execute_delegate(
            deps,
            env,
            info,
            api.addr_validate(&validator)?,
            min_duration,
        ),
        ExecuteMsg::Undelegate { validator, amount } => {
            execute_undelegate(deps, env, info, api.addr_validate(&validator)?, amount)
        }
        ExecuteMsg::Redelegate {
            src_validator,
            dst_validator,
            amount,
        } => execute_redelegate(
            deps,
            info,
            api.addr_validate(&src_validator)?,
            api.addr_validate(&dst_validator)?,
            amount,
        ),
        ExecuteMsg::Claim { validator } => {
            execute_claim(deps, info, api.addr_validate(&validator)?)
        }
    }
}

pub fn execute_delegate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    validator: Addr,
    min_duration: u64,
) -> Result<Response, ContractError> {
    let amount = must_pay(&info, NATIVE_DENOM)?;

    // deps.querier.query_delegation(delegator, validator)

    STAKE.update(
        deps.storage,
        (&info.sender, &validator),
        |existing_stake| -> Result<_, ContractError> {
            match existing_stake {
                Some(stake) => Ok(Stake {
                    amount: stake.amount + amount,
                    end_time: stake.end_time,
                }),
                None => Ok(Stake {
                    amount,
                    end_time: env.block.time.plus_seconds(min_duration * 24 * 60 * 60),
                }),
            }
        },
    )?;

    let stake_msg = CosmosMsg::Staking(StakingMsg::Delegate {
        validator: validator.to_string(),
        amount: coin(amount.u128(), NATIVE_DENOM),
    });

    let set_withdraw_address_msg = CosmosMsg::Distribution(DistributionMsg::SetWithdrawAddress {
        address: info.sender.to_string(),
    });

    Ok(Response::default()
        .add_attribute("action", "delegate")
        .add_attribute("validator", validator)
        .add_messages(vec![stake_msg, set_withdraw_address_msg]))
}

pub fn execute_undelegate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    validator: Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    STAKE.update(
        deps.storage,
        (&info.sender, &validator),
        |existing_stake| -> Result<_, ContractError> {
            match existing_stake {
                None => Err(ContractError::DelegationNotFound {}),
                Some(stake) => {
                    if stake.end_time < env.block.time {
                        return Err(ContractError::MinDurationNotPassed {});
                    }
                    Ok(Stake {
                        amount: stake.amount - amount,
                        end_time: stake.end_time,
                    })
                }
            }
        },
    )?;

    if STAKE.load(deps.storage, (&info.sender, &validator))?.amount == Uint128::zero() {
        STAKE.remove(deps.storage, (&info.sender, &validator));
    }

    let undelegate_msg = CosmosMsg::Staking(StakingMsg::Undelegate {
        validator: validator.to_string(),
        amount: coin(amount.u128(), NATIVE_DENOM),
    });

    Ok(Response::default()
        .add_attribute("action", "undelegate")
        .add_attribute("validator", validator)
        .add_message(undelegate_msg))
}

pub fn execute_redelegate(
    deps: DepsMut,
    info: MessageInfo,
    src_validator: Addr,
    dst_validator: Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    // 1. check if src delegation exists -- if not, return error
    // 2. subtract amount from src delegation
    // 3. check if dst delegation exists with the same time -- if so, add amount to it
    // 4. if not, create new dst delegation with the same `end_time` as src delegation
    // 5. remove src delegation if amount is zero

    // 1 + 2
    let mut src_stake = STAKE.load(deps.storage, (&info.sender, &src_validator))?;
    src_stake.amount -= amount;

    // 3 + 4
    let mut dst_stake = STAKE.load(deps.storage, (&info.sender, &dst_validator))?;
    if dst_stake.end_time == src_stake.end_time {
        dst_stake.amount += amount;
        STAKE.save(deps.storage, (&info.sender, &dst_validator), &dst_stake)?;
    } else {
        dst_stake = Stake {
            amount,
            end_time: src_stake.end_time,
        };
        STAKE.save(deps.storage, (&info.sender, &dst_validator), &dst_stake)?;
    }

    // 5
    if src_stake.amount == Uint128::zero() {
        STAKE.remove(deps.storage, (&info.sender, &src_validator));
    }

    let redelegate_msg = CosmosMsg::Staking(StakingMsg::Redelegate {
        src_validator: src_validator.to_string(),
        dst_validator: dst_validator.to_string(),
        amount: coin(amount.u128(), NATIVE_DENOM),
    });

    Ok(Response::default()
        .add_attribute("action", "redelegate")
        .add_attribute("src_validator", src_validator)
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
