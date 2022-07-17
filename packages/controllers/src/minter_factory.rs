use cosmwasm_std::{ensure_eq, Addr, Deps, DepsMut, Reply, StdError, StdResult};
use cw_storage_plus::Map;
use cw_utils::parse_reply_instantiate_data;
use minters::{Minter, MinterParams, MinterStatusResponse, UpdateParamsMsg};
use sg_std::{Response, NATIVE_DENOM};
use thiserror::Error;

pub const MINTERS: Map<&Addr, Minter> = Map::new("m");

#[derive(Error, Debug, PartialEq)]
pub enum MinterFactoryError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("InstantiateMinterError")]
    InstantiateMinterError {},

    #[error("InvalidDenom")]
    InvalidDenom {},
}

/// Only governance can update contract params
pub fn upsert_minter_status(
    deps: DepsMut,
    minter: String,
    verified: bool,
    blocked: bool,
) -> StdResult<Response> {
    let minter_addr = deps.api.addr_validate(&minter)?;

    let _: StdResult<Minter> = MINTERS.update(deps.storage, &minter_addr, |m| match m {
        None => Ok(Minter { verified, blocked }),
        Some(mut m) => {
            m.verified = verified;
            m.blocked = blocked;
            Ok(m)
        }
    });

    Ok(Response::new().add_attribute("action", "sudo_update_minter_status"))
}

pub fn handle_reply(deps: DepsMut, msg: Reply) -> Result<Response, MinterFactoryError> {
    let reply = parse_reply_instantiate_data(msg);

    match reply {
        Ok(res) => {
            let minter = res.contract_address;
            upsert_minter_status(deps, minter, false, false)?;
            Ok(Response::default().add_attribute("action", "instantiate_minter_reply"))
        }
        Err(_) => Err(MinterFactoryError::InstantiateMinterError {}),
    }
}

pub fn update_params<T, C>(
    params: &mut MinterParams<C>,
    param_msg: UpdateParamsMsg<T>,
) -> Result<(), MinterFactoryError> {
    params.code_id = param_msg.code_id.unwrap_or(params.code_id);

    if let Some(creation_fee) = param_msg.creation_fee {
        ensure_eq!(
            &creation_fee.denom,
            &NATIVE_DENOM,
            MinterFactoryError::InvalidDenom {}
        );
        params.creation_fee = creation_fee;
    }

    params.max_token_limit = param_msg.max_token_limit.unwrap_or(params.max_token_limit);
    params.max_per_address_limit = param_msg
        .max_per_address_limit
        .unwrap_or(params.max_per_address_limit);

    if let Some(min_mint_price) = param_msg.min_mint_price {
        ensure_eq!(
            &min_mint_price.denom,
            &NATIVE_DENOM,
            MinterFactoryError::InvalidDenom {}
        );
        params.min_mint_price = min_mint_price;
    }

    if let Some(airdrop_mint_price) = param_msg.airdrop_mint_price {
        ensure_eq!(
            &airdrop_mint_price.denom,
            &NATIVE_DENOM,
            MinterFactoryError::InvalidDenom {}
        );
        params.airdrop_mint_price = airdrop_mint_price;
    }

    params.mint_fee_bps = param_msg.mint_fee_bps.unwrap_or(params.mint_fee_bps);
    params.airdrop_mint_fee_bps = param_msg
        .airdrop_mint_fee_bps
        .unwrap_or(params.airdrop_mint_fee_bps);

    Ok(())
}

pub fn query_minter_status(deps: Deps, minter_addr: String) -> StdResult<MinterStatusResponse> {
    let minter = MINTERS.load(deps.storage, &deps.api.addr_validate(&minter_addr)?)?;

    Ok(MinterStatusResponse { minter })
}
