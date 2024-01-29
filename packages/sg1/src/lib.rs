use anybuf::Anybuf;
use cosmwasm_std::{
    coin, coins, Addr, BankMsg, Coin, CosmosMsg, Decimal, Event, MessageInfo, Response, SubMsg,
    Uint128,
};
use cw_utils::{may_pay, PaymentError};
use sg_std::NATIVE_DENOM;
use thiserror::Error;
// governance parameters
const FEE_BURN_PERCENT: u64 = 50;
const FOUNDATION: &str = "stars1xqz6xujjyz0r9uzn7srasle5uynmpa0zkjr5l8";

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
        fair_burn(info.sender.to_string(), fee, developer, res);
    }

    Ok(())
}

/// IBC assets go to community pool and dev
/// 7/29/23 temporary fix until we switch to using fairburn contract
pub fn ibc_denom_fair_burn(
    fee: Coin,
    developer: Option<Addr>,
    res: &mut Response,
) -> Result<(), FeeError> {
    let mut event = Event::new("ibc-fair-burn");

    match &developer {
        Some(developer) => {
            // Calculate the fees. 50% to dev, 50% to foundation
            let dev_fee = (fee.amount.mul_ceil(Decimal::percent(FEE_BURN_PERCENT))).u128();
            let dev_coin = coin(dev_fee, fee.denom.to_string());
            let foundation_coin = coin(fee.amount.u128() - dev_fee, fee.denom);

            event = event.add_attribute("dev_addr", developer.to_string());
            event = event.add_attribute("dev_coin", dev_coin.to_string());
            event = event.add_attribute("foundation_coin", foundation_coin.to_string());

            res.messages.push(SubMsg::new(BankMsg::Send {
                to_address: developer.to_string(),
                amount: vec![dev_coin],
            }));
            res.messages.push(SubMsg::new(BankMsg::Send {
                to_address: FOUNDATION.to_string(),
                amount: vec![foundation_coin],
            }));
        }
        None => {
            // No dev, send all to foundation.
            event = event.add_attribute("foundation_coin", fee.to_string());
            res.messages.push(SubMsg::new(BankMsg::Send {
                to_address: FOUNDATION.to_string(),
                amount: vec![fee],
            }));
        }
    }

    res.events.push(event);
    Ok(())
}

/// Burn and distribute fees, assuming the right fee is passed in
pub fn fair_burn(sender: String, fee: u128, developer: Option<Addr>, res: &mut Response) {
    let mut event = Event::new("fair-burn");

    // calculate the fair burn fee
    let burn_fee = (Uint128::from(fee) * Decimal::percent(FEE_BURN_PERCENT)).u128();
    let burn_coin = coins(burn_fee, NATIVE_DENOM);
    res.messages
        .push(SubMsg::new(BankMsg::Burn { amount: burn_coin }));
    event = event.add_attribute("burn_amount", Uint128::from(burn_fee).to_string());

    // send remainder to developer or community pool
    let remainder = fee - burn_fee;

    if let Some(dev) = developer {
        res.messages.push(SubMsg::new(BankMsg::Send {
            to_address: dev.to_string(),
            amount: coins(remainder, NATIVE_DENOM),
        }));
        event = event.add_attribute("dev", dev.to_string());
        event = event.add_attribute("dev_amount", Uint128::from(remainder).to_string());
    } else {
        res.messages.push(SubMsg::new(create_fund_fairburn_pool_msg(
            sender,
            &coin(remainder, NATIVE_DENOM),
        )));
        event = event.add_attribute("dist_amount", Uint128::from(remainder).to_string());
    }

    res.events.push(event);
}

fn create_fund_fairburn_pool_msg(sender: String, amount: &Coin) -> CosmosMsg {
    CosmosMsg::Stargate {
        type_url: "/publicawesome.stargaze.alloc.v1beta1.MsgFundFairburnPool".to_string(),
        value: encode_msg_fund_fairburn_pool(sender, amount).into(),
    }
}
/// Encode the message to fund the fairburn pool
/// following the protobuf spec in
/// https://github.com/public-awesome/stargaze/blob/efdb9212e037e05fc429c0cfbcf425ad11855e15/proto/publicawesome/stargaze/alloc/v1beta1/tx.proto#L49
fn encode_msg_fund_fairburn_pool(sender: String, amount: &Coin) -> Vec<u8> {
    let coin = Anybuf::new()
        .append_string(1, &amount.denom)
        .append_string(2, amount.amount.to_string());
    Anybuf::new()
        .append_string(1, sender)
        .append_message(2, &coin)
        .into_vec()
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
    use crate::create_fund_fairburn_pool_msg;
    use cosmwasm_std::{coin, coins, Addr, BankMsg, Response};
    use sg_std::NATIVE_DENOM;

    use crate::{fair_burn, SubMsg};

    #[test]
    fn check_fair_burn_no_dev_rewards() {
        let mut res = Response::new();

        fair_burn(Addr::unchecked("sender").to_string(), 9u128, None, &mut res);
        let burn_msg = SubMsg::new(BankMsg::Burn {
            amount: coins(4, "ustars".to_string()),
        });
        let dist_msg = SubMsg::new(create_fund_fairburn_pool_msg(
            Addr::unchecked("sender").to_string(),
            &coin(5, NATIVE_DENOM),
        ));
        assert_eq!(res.messages.len(), 2);
        assert_eq!(res.messages[0], burn_msg);
        assert_eq!(res.messages[1], dist_msg);
    }

    #[test]
    fn check_fair_burn_with_dev_rewards() {
        let mut res = Response::new();

        fair_burn(
            Addr::unchecked("sender").to_string(),
            9u128,
            Some(Addr::unchecked("geordi")),
            &mut res,
        );
        let bank_msg = SubMsg::new(BankMsg::Send {
            to_address: "geordi".to_string(),
            amount: coins(5, NATIVE_DENOM),
        });
        let burn_msg = SubMsg::new(BankMsg::Burn {
            amount: coins(4, NATIVE_DENOM),
        });
        assert_eq!(res.messages.len(), 2);
        assert_eq!(res.messages[0], burn_msg);
        assert_eq!(res.messages[1], bank_msg);
    }

    #[test]
    fn check_fair_burn_with_dev_rewards_different_amount() {
        let mut res = Response::new();

        fair_burn(
            Addr::unchecked("sender").to_string(),
            1420u128,
            Some(Addr::unchecked("geordi")),
            &mut res,
        );
        let bank_msg = SubMsg::new(BankMsg::Send {
            to_address: "geordi".to_string(),
            amount: coins(710, NATIVE_DENOM),
        });
        let burn_msg = SubMsg::new(BankMsg::Burn {
            amount: coins(710, NATIVE_DENOM),
        });
        assert_eq!(res.messages.len(), 2);
        assert_eq!(res.messages[0], burn_msg);
        assert_eq!(res.messages[1], bank_msg);
    }
}
