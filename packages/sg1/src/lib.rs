use cosmwasm_std::{coins, Addr, BankMsg, Decimal, Event, MessageInfo, Uint128};
use cw_utils::{may_pay, PaymentError};
use sg_std::{create_fund_fairburn_pool_msg, Response, SubMsg, NATIVE_DENOM};
use thiserror::Error;

// governance parameters
const FEE_BURN_PERCENT: u64 = 50;
const DEV_INCENTIVE_PERCENT: u64 = 50;

/// Burn and distribute fees and return an error if the fee is not enough
pub fn checked_fair_burn(
    info: &MessageInfo,
    fee: u128,
    developer: Option<Addr>,
    res: &mut Response,
) -> Result<(), FeeError> {
    // Use may_pay because fees could be 0. Add check to avoid transferring 0 funds
    let payment = may_pay(info, NATIVE_DENOM)?;
    if payment.u128() < fee {
        return Err(FeeError::InsufficientFee(fee, payment.u128()));
    };

    if payment.u128() != 0u128 {
        fair_burn(fee, developer, res);
    }

    Ok(())
}

/// Burn and distribute fees, assuming the right fee is passed in
pub fn fair_burn(fee: u128, developer: Option<Addr>, res: &mut Response) {
    let mut event = Event::new("fair-burn");

    let (burn_percent, dev_fee) = match developer {
        Some(dev) => {
            let dev_fee = (Uint128::from(fee) * Decimal::percent(DEV_INCENTIVE_PERCENT)).u128();
            res.messages.push(SubMsg::new(BankMsg::Send {
                to_address: dev.to_string(),
                amount: coins(dev_fee, NATIVE_DENOM),
            }));
            event = event.add_attribute("dev", dev.to_string());
            event = event.add_attribute("dev_amount", Uint128::from(dev_fee).to_string());
            (
                Decimal::percent(FEE_BURN_PERCENT - DEV_INCENTIVE_PERCENT),
                dev_fee,
            )
        }
        None => (Decimal::percent(FEE_BURN_PERCENT), 0u128),
    };

    // burn half the fee
    let burn_fee = (Uint128::from(fee) * burn_percent).u128();
    let burn_coin = coins(burn_fee, NATIVE_DENOM);
    res.messages
        .push(SubMsg::new(BankMsg::Burn { amount: burn_coin }));

    // Send other half to fairburn pool
    let dist_amount = fee - (burn_fee + dev_fee);
    res.messages
        .push(SubMsg::new(create_fund_fairburn_pool_msg(coins(
            dist_amount,
            NATIVE_DENOM,
        ))));

    event = event.add_attribute("burn_amount", Uint128::from(burn_fee).to_string());
    event = event.add_attribute("dist_amount", Uint128::from(dist_amount).to_string());
    res.events.push(event);
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum FeeError {
    #[error("Insufficient fee: expected {0}, got {1}")]
    InsufficientFee(u128, u128),

    #[error("{0}")]
    Payment(#[from] PaymentError),
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{coins, Addr, BankMsg};
    use sg_std::{create_fund_fairburn_pool_msg, Response, NATIVE_DENOM};

    use crate::{fair_burn, SubMsg};

    #[test]
    fn check_fair_burn_no_dev_rewards() {
        let mut res = Response::new();

        fair_burn(1000u128, None, &mut res);
        let burn_msg = SubMsg::new(BankMsg::Burn {
            amount: coins(500, "ustars".to_string()),
        });
        let dist_msg = SubMsg::new(create_fund_fairburn_pool_msg(coins(500, NATIVE_DENOM)));
        assert_eq!(res.messages.len(), 2);
        assert_eq!(res.messages[0], burn_msg);
        assert_eq!(res.messages[1], dist_msg);
    }

    #[test]
    fn check_fair_burn_with_dev_rewards() {
        let mut res = Response::new();

        fair_burn(1000u128, Some(Addr::unchecked("geordi")), &mut res);
        let bank_msg = SubMsg::new(BankMsg::Send {
            to_address: "geordi".to_string(),
            amount: coins(500, NATIVE_DENOM),
        });
        let burn_msg = SubMsg::new(BankMsg::Burn {
            amount: coins(0, NATIVE_DENOM),
        });
        let dist_msg = SubMsg::new(create_fund_fairburn_pool_msg(coins(500, NATIVE_DENOM)));
        assert_eq!(res.messages.len(), 3);
        assert_eq!(res.messages[0], bank_msg);
        assert_eq!(res.messages[1], burn_msg);
        assert_eq!(res.messages[2], dist_msg);
    }
}
