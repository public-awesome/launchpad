#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_binary, Addr, BalanceResponse, BankMsg, Binary, CosmosMsg, Deps, DepsMut,
    DistributionMsg, Env, MessageInfo, Response, StakingMsg, StdResult, WasmMsg,
};
use cw2::set_contract_version;
use cw_utils::{must_pay, nonpayable};
use sg_std::NATIVE_DENOM;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, StakeResponse};
use crate::state::{Stake, STAKE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg-timelocked-stake";
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
        owner: info.sender.clone(),
        validator: deps.api.addr_validate(&msg.validator)?,
        end_time: env.block.time.plus_seconds(msg.min_duration),
        amount: must_pay(&info, NATIVE_DENOM)?,
        min_withdrawal: msg.min_withdrawal,
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
        ExecuteMsg::Claim {} => execute_claim(deps, env, info),
        ExecuteMsg::Reinvest {} => execute_reinvest(deps, env, info),
        ExecuteMsg::_Delegate {} => _execute_delegate(deps, env, info),
    }
}

pub fn execute_undelegate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let stake = STAKE.load(deps.storage)?;
    if stake.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    STAKE.remove(deps.storage);

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
    if stake.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let redelegate_msg = CosmosMsg::Staking(StakingMsg::Redelegate {
        src_validator: stake.validator.to_string(),
        dst_validator: dst_validator.to_string(),
        amount: coin(stake.amount.u128(), NATIVE_DENOM),
    });

    stake.validator = dst_validator.clone();
    STAKE.save(deps.storage, &stake)?;

    Ok(Response::default()
        .add_attribute("action", "redelegate")
        .add_attribute("dst_validator", dst_validator)
        .add_message(redelegate_msg))
}

pub fn execute_claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let stake = STAKE.load(deps.storage)?;
    if stake.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let balance = deps
        .querier
        .query_balance(&env.contract.address, NATIVE_DENOM)?;
    if balance.amount < stake.min_withdrawal {
        return Err(ContractError::BalanceTooSmall {});
    }

    Ok(Response::default()
        .add_attribute("action", "claim")
        .add_message(BankMsg::Send {
            to_address: stake.owner.to_string(),
            amount: vec![coin(balance.amount.u128(), NATIVE_DENOM)],
        }))
}

/// Reinvest withdraws all pending rewards, then issues a callback to itself to delegate.
/// It reinvests new earnings (and anything else that accumulated)
pub fn execute_reinvest(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let stake = STAKE.load(deps.storage)?;
    if stake.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let res = Response::new()
        .add_message(DistributionMsg::WithdrawDelegatorReward {
            validator: stake.validator.to_string(),
        })
        .add_message(WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: to_binary(&ExecuteMsg::_Delegate {})?,
            funds: vec![],
        });
    Ok(res)
}

pub fn _execute_delegate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // this is just meant as a call-back to ourself
    if info.sender != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }

    let balance = deps
        .querier
        .query_balance(&env.contract.address, NATIVE_DENOM)?;

    let mut stake = STAKE.load(deps.storage)?;
    stake.amount += balance.amount;
    STAKE.save(deps.storage, &stake)?;

    let res = Response::new()
        .add_message(StakingMsg::Delegate {
            validator: stake.validator.to_string(),
            amount: balance.clone(),
        })
        .add_attribute("action", "reinvest")
        .add_attribute("amount", balance.amount);
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Stake {} => to_binary(&query_stake(deps)?),
        QueryMsg::Balance {} => to_binary(&query_balance(deps, env)?),
    }
}

fn query_stake(deps: Deps) -> StdResult<StakeResponse> {
    let stake = STAKE.load(deps.storage)?;

    Ok(StakeResponse { stake })
}

fn query_balance(deps: Deps, env: Env) -> StdResult<BalanceResponse> {
    let amount = deps
        .querier
        .query_balance(&env.contract.address, NATIVE_DENOM)?;

    Ok(BalanceResponse { amount })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::coins;
    use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};

    #[test]
    fn proper_initialization() {
        // let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        // let msg = InstantiateMsg {};
        // let info = mock_info("creator", &coins(1000, "earth"));

        // // we can just call .unwrap() to assert this was a success
        // let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        // assert_eq!(0, res.messages.len());

        // // it worked, let's query the state
        // let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        // let value: CountResponse = from_binary(&res).unwrap();
        // assert_eq!(17, value.count);
    }

    #[test]
    fn increment() {
        // let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        // let msg = InstantiateMsg {};
        // let info = mock_info("creator", &coins(2, "token"));
        // let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // // beneficiary can release it
        // let info = mock_info("anyone", &coins(2, "token"));
        // let msg = ExecuteMsg::Increment {};
        // let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // // should increase counter by 1
        // let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        // let value: CountResponse = from_binary(&res).unwrap();
        // assert_eq!(18, value.count);
    }
}
