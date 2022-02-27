use crate::{create_fund_community_pool_msg, StargazeMsgWrapper, NATIVE_DENOM};
use cosmwasm_std::{coin, coins, BankMsg, Coin, CosmosMsg, Decimal, Env, MessageInfo, Uint128};
use cw_utils::{may_pay, PaymentError};
use thiserror::Error;

// governance parameters
const FEE_BURN_PERCENT: u64 = 50;

// deal with zero and non-zero coin amounts for msgs
fn convert_coins_for_msg(msg_coin: Coin) -> Vec<Coin> {
    if msg_coin.amount > Uint128::zero() {
        vec![msg_coin]
    } else {
        println!("sg-std fees: no funds sent");
        coins(0, NATIVE_DENOM)
    }
}

type SubMsg = CosmosMsg<StargazeMsgWrapper>;
pub fn burn_and_distribute_fee(
    _env: Env,
    info: &MessageInfo,
    fee_amount: u128,
) -> Result<Vec<SubMsg>, FeeError> {
    println!("inside burn and distribute fee");
    let payment = may_pay(info, NATIVE_DENOM)?;
    if payment.u128() < fee_amount {
        return Err(FeeError::InsufficientFee(fee_amount, payment.u128()));
    };

    println!("before burn calc");
    // calculate the fee to burn
    let burn_percent = Decimal::percent(FEE_BURN_PERCENT);
    let burn_fee = Uint128::from(fee_amount) * burn_percent;
    let burn_coin = coin(burn_fee.u128(), NATIVE_DENOM);
    // burn half the fee
    println!(
        "fee burn msg: {:?}",
        convert_coins_for_msg(burn_coin.clone())
    );
    let fee_burn_msg = BankMsg::Burn {
        amount: convert_coins_for_msg(burn_coin),
    };
    println!("fee burn msg: {:?}", fee_burn_msg);

    // Send other half to community pool
    let fund_community_pool_msg = create_fund_community_pool_msg(convert_coins_for_msg(coin(
        fee_amount - burn_fee.u128(),
        NATIVE_DENOM,
    )));

    println!("fund community pool msg: {:?}", fund_community_pool_msg);

    Ok(vec![CosmosMsg::Bank(fee_burn_msg), fund_community_pool_msg])
}

#[derive(Error, Debug, PartialEq)]
pub enum FeeError {
    #[error("Insufficient fee: expected {0}, got {1}")]
    InsufficientFee(u128, u128),

    #[error("{0}")]
    Payment(#[from] PaymentError),
}
