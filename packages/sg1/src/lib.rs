use anybuf::Anybuf;
use cosmwasm_std::{
    coin, coins, ensure, Addr, BankMsg, Coin, CosmosMsg, Decimal, Env, Event, MessageInfo,
    Response, SubMsg, Uint128,
};
use cw_utils::{may_pay, must_pay, PaymentError};
use sg_utils::NATIVE_DENOM;
use thiserror::Error;

// governance parameters
const FEE_BURN_PERCENT: u64 = 50;
const FOUNDATION: &str = "init1d7q84m8y8gy0dcql090sqg9h7w9kydzntzx5yk";
const LAUNCHPAD_DAO_ADDRESS: &str = "init1flaytkt2zyylnc2p9u77jjgwrct206x86m03ac";
const LIQUIDITY_DAO_ADDRESS: &str = "init176qen2pg7lmfel2ph4rrsmm04qtl8qv67mj84d";

/// Burn and distribute fees and return an error if the fee is not enough
pub fn checked_fair_burn(
    info: &MessageInfo,
    env: &Env,
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
        fair_burn(env.contract.address.to_string(), fee, developer, res);
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

pub fn distribute_mint_fees(
    fee: Coin,
    res: &mut Response,
    is_featured: bool,
    developer: Option<Addr>,
) -> Result<(), FeeError> {
    let liquidity_dao_ratio: Decimal = Decimal::from_ratio(1u128, 5u128);
    let liquidity_dao_ratio_featured: Decimal = Decimal::from_ratio(1u128, 8u128);

    let mut event = Event::new("mint-fee-distribution");

    let liquidity_dao_percentage = if is_featured {
        liquidity_dao_ratio_featured
    } else {
        liquidity_dao_ratio
    };

    match &developer {
        Some(developer) => {
            let dev_fee = fee
                .amount
                .mul_ceil(Decimal::percent(FEE_BURN_PERCENT))
                .u128();
            let dev_coin = coin(dev_fee, fee.denom.to_string());
            let remaining_coin = coin(fee.amount.u128() - dev_fee, fee.denom.clone());

            let liquidity_dao_fee =
                (remaining_coin.amount.mul_ceil(liquidity_dao_percentage)).u128();
            let liquidity_dao_coin = coin(liquidity_dao_fee, fee.denom.to_string());
            let launchpad_dao_coin =
                coin(remaining_coin.amount.u128() - liquidity_dao_fee, fee.denom);

            event = event.add_attribute("dev_addr", developer.to_string());
            event = event.add_attribute("dev_coin", dev_coin.to_string());
            event = event.add_attribute("liquidity_DAO_addr", LIQUIDITY_DAO_ADDRESS.to_string());
            event = event.add_attribute("liquidity_DAO_coin", liquidity_dao_coin.to_string());
            event = event.add_attribute("launchpad_DAO_addr", LAUNCHPAD_DAO_ADDRESS.to_string());
            event = event.add_attribute("launchpad_DAO_coin", launchpad_dao_coin.to_string());

            res.messages.push(SubMsg::new(BankMsg::Send {
                to_address: developer.to_string(),
                amount: vec![dev_coin],
            }));
            res.messages.push(SubMsg::new(BankMsg::Send {
                to_address: LIQUIDITY_DAO_ADDRESS.to_string(),
                amount: vec![liquidity_dao_coin],
            }));
            res.messages.push(SubMsg::new(BankMsg::Send {
                to_address: LAUNCHPAD_DAO_ADDRESS.to_string(),
                amount: vec![launchpad_dao_coin],
            }));
        }
        None => {
            let liquidity_dao_fee = fee.amount.mul_ceil(liquidity_dao_percentage).u128();
            let liquidity_dao_coin = coin(liquidity_dao_fee, fee.denom.to_string());
            let launchpad_dao_coin = coin(fee.amount.u128() - liquidity_dao_fee, fee.denom);

            event = event.add_attribute("liquidity_DAO_addr", LIQUIDITY_DAO_ADDRESS.to_string());
            event = event.add_attribute("liquidity_DAO_coin", liquidity_dao_coin.to_string());
            event = event.add_attribute("launchpad_DAO_addr", LAUNCHPAD_DAO_ADDRESS.to_string());
            event = event.add_attribute("launchpad_DAO_coin", launchpad_dao_coin.to_string());

            res.messages.push(SubMsg::new(BankMsg::Send {
                to_address: LIQUIDITY_DAO_ADDRESS.to_string(),
                amount: vec![liquidity_dao_coin],
            }));
            res.messages.push(SubMsg::new(BankMsg::Send {
                to_address: LAUNCHPAD_DAO_ADDRESS.to_string(),
                amount: vec![launchpad_dao_coin],
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
        let msg_fund_fairburn_pool =
            create_fund_fairburn_pool_msg(sender, &coin(remainder, NATIVE_DENOM));
        res.messages.push(SubMsg::new(msg_fund_fairburn_pool));
        event = event.add_attribute("dist_amount", Uint128::from(remainder).to_string());
    }

    res.events.push(event);
}

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

fn create_fund_fairburn_pool_msg(sender: String, amount: &Coin) -> CosmosMsg {
    CosmosMsg::Stargate {
        type_url: "/publicawesome.stargaze.alloc.v1beta1.MsgFundFairburnPool".to_string(),
        value: encode_msg_fund_fairburn_pool(sender, amount).into(),
    }
}
pub fn transfer_funds_to_launchpad_dao(
    info: &MessageInfo,
    fee: u128,
    accepted_denom: &str,
    res: &mut Response,
) -> Result<(), FeeError> {
    let payment = must_pay(info, accepted_denom)?;
    ensure!(
        payment.u128() >= fee,
        FeeError::InsufficientFee(fee, payment.u128())
    );

    let msg = BankMsg::Send {
        to_address: LAUNCHPAD_DAO_ADDRESS.to_string(),
        amount: vec![coin(payment.u128(), accepted_denom)],
    };
    res.messages.push(SubMsg::new(msg));

    Ok(())
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
    use cosmwasm_std::{coin, coins, Addr, BankMsg, Response};
    use sg_utils::NATIVE_DENOM;

    use crate::{create_fund_fairburn_pool_msg, fair_burn, SubMsg};

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
