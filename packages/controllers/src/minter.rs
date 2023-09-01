use cosmwasm_std::{coin, BankMsg, Coin, MessageInfo, Uint128};
use cosmwasm_std::{Addr, StdError};
use cw_utils::may_pay;
use sg_std::Response;

pub fn compute_seller_amount(
    mut res: Response,
    mint_price_with_discounts: Coin,
    network_fee: Uint128,
    payment_address: Option<Addr>,
    admin: Addr,
) -> Result<(Response, Uint128), StdError> {
    let seller_amount = {
        // the net amount is mint price - network fee (mint free + dev fee)
        let amount = mint_price_with_discounts.amount.checked_sub(network_fee)?;
        let payment_address = payment_address;
        let seller = admin;
        // Sending 0 coins fails, so only send if amount is non-zero
        if !amount.is_zero() {
            let msg = BankMsg::Send {
                to_address: payment_address.unwrap_or(seller).to_string(),
                amount: vec![coin(amount.u128(), mint_price_with_discounts.denom)],
            };
            res = res.clone().add_message(msg);
        }
        amount
    };
    Ok((res, seller_amount))
}

pub fn pay_mint(
    info: MessageInfo,
    mint_price_with_discounts: Coin,
    config_denom: String,
) -> Result<Uint128, StdError> {
    let payment =
        may_pay(&info, &config_denom).map_err(|e| StdError::GenericErr { msg: e.to_string() })?;
    if payment != mint_price_with_discounts.amount {
        let err_msg = format!(
            "IncorrectPaymentAmount {0} != {1}",
            coin(payment.u128(), &config_denom),
            mint_price_with_discounts,
        );
        return Err(StdError::GenericErr { msg: err_msg });
    }
    Ok(payment)
}
