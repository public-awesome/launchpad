use crate::{create_fund_community_pool_msg, StargazeMsgWrapper, NATIVE_DENOM};
use cosmwasm_std::{coin, coins, BankMsg, CosmosMsg, Decimal, Env, MessageInfo, Uint128};
use cw_utils::{must_pay, PaymentError};
use thiserror::Error;

// governance parameters
const FEE_BURN_PERCENT: u64 = 50;

type SubMsg = CosmosMsg<StargazeMsgWrapper>;
pub fn burn_and_distribute_fee(
    _env: Env,
    info: &MessageInfo,
    fee_amount: u128,
) -> Result<Vec<SubMsg>, FeeError> {
    must_pay(info, NATIVE_DENOM)?;

    // calculate the fee to burn
    let burn_percent = Decimal::percent(FEE_BURN_PERCENT);
    let burn_fee = Uint128::from(fee_amount) * burn_percent;
    let burn_coin = coin(burn_fee.u128(), NATIVE_DENOM);
    // burn half the fee
    let fee_burn_msg = BankMsg::Burn {
        amount: vec![burn_coin],
    };

    // Send other half to community pool
    let fund_community_pool_msg =
        create_fund_community_pool_msg(coins(fee_amount - burn_fee.u128(), NATIVE_DENOM));

    Ok(vec![CosmosMsg::Bank(fee_burn_msg), fund_community_pool_msg])
}

#[derive(Error, Debug, PartialEq)]
pub enum FeeError {
    #[error("{0}")]
    Payment(#[from] PaymentError),
}
