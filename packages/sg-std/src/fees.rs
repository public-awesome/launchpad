use crate::{create_fund_community_pool_msg, StargazeMsgWrapper, NATIVE_DENOM};
use cosmwasm_std::{coin, coins, BankMsg, CosmosMsg, Decimal, Env, MessageInfo};
use cw_utils::{must_pay, PaymentError};
use thiserror::Error;

const FEE_BURN_PERCENT: u64 = 50;

pub fn handle_fee(
    env: Env,
    info: &MessageInfo,
    expected_fee: u128,
) -> Result<Vec<CosmosMsg<StargazeMsgWrapper>>, FeeError> {
    let payment = must_pay(info, NATIVE_DENOM)?;
    if payment.u128() != expected_fee {
        return Err(FeeError::InvalidFee {});
    }

    // calculate the fee to burn
    let burn_percent = Decimal::percent(FEE_BURN_PERCENT);
    let burn_fee = payment * burn_percent;
    let burn_coin = coin(burn_fee.u128(), NATIVE_DENOM);
    // send fee to contract to be burned
    let send_fee_msg = BankMsg::Send {
        to_address: env.contract.address.to_string(),
        amount: vec![burn_coin.clone()],
    };
    // burn half the fee
    let fee_burn_msg = BankMsg::Burn {
        amount: vec![burn_coin],
    };

    let fund_community_pool_msg =
        create_fund_community_pool_msg(coins(payment.u128() - burn_fee.u128(), NATIVE_DENOM));

    Ok(vec![
        CosmosMsg::Bank(send_fee_msg),
        CosmosMsg::Bank(fee_burn_msg),
        fund_community_pool_msg,
    ])
}

#[derive(Error, Debug, PartialEq)]
pub enum FeeError {
    #[error("InvalidFee")]
    InvalidFee {},

    #[error("{0}")]
    Payment(#[from] PaymentError),
}
