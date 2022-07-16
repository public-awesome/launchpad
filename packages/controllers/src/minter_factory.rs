use cosmwasm_std::{Addr, DepsMut, Env, Reply, StdError, StdResult};
use cw_storage_plus::Map;
use cw_utils::parse_reply_instantiate_data;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sg_std::Response;
use thiserror::Error;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Minter {
    pub verified: bool,
    pub blocked: bool,
}

pub const MINTERS: Map<&Addr, Minter> = Map::new("m");

#[derive(Error, Debug, PartialEq)]
pub enum MinterFactoryError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("InstantiateMinterError")]
    InstantiateMinterError {},
}

/// Only governance can update contract params
pub fn upsert_minter_status(
    deps: DepsMut,
    _env: Env,
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
            MINTERS.save(
                deps.storage,
                &Addr::unchecked(res.contract_address),
                &Minter {
                    verified: false,
                    blocked: false,
                },
            )?;
            Ok(Response::default().add_attribute("action", "instantiate_minter_reply"))
        }
        Err(_) => Err(MinterFactoryError::InstantiateMinterError {}),
    }
}

// pub fn update_params(deps: DepsMut, param_msg: UpdateParamsMsg) -> Result<Response, ContractError> {
//     let mut params = SUDO_PARAMS.load(deps.storage)?;
//     let native_denom = deps.querier.query_bonded_denom()?;

//     params.code_id = param_msg.code_id.unwrap_or(params.code_id);

//     if let Some(creation_fee) = param_msg.creation_fee {
//         ensure_eq!(
//             &creation_fee.denom,
//             &native_denom,
//             ContractError::InvalidDenom {}
//         );
//         params.creation_fee = creation_fee;
//     }

//     params.max_token_limit = param_msg.max_token_limit.unwrap_or(params.max_token_limit);
//     params.max_per_address_limit = param_msg
//         .max_per_address_limit
//         .unwrap_or(params.max_per_address_limit);

//     if let Some(min_mint_price) = param_msg.min_mint_price {
//         ensure_eq!(
//             &min_mint_price.denom,
//             &native_denom,
//             ContractError::InvalidDenom {}
//         );
//         params.min_mint_price = min_mint_price;
//     }

//     if let Some(airdrop_mint_price) = param_msg.airdrop_mint_price {
//         ensure_eq!(
//             &airdrop_mint_price.denom,
//             &native_denom,
//             ContractError::InvalidDenom {}
//         );
//         params.airdrop_mint_price = airdrop_mint_price;
//     }

//     params.mint_fee_bps = param_msg.mint_fee_bps.unwrap_or(params.mint_fee_bps);
//     params.airdrop_mint_fee_bps = param_msg
//         .airdrop_mint_fee_bps
//         .unwrap_or(params.airdrop_mint_fee_bps);

//     if let Some(shuffle_fee) = param_msg.shuffle_fee {
//         ensure_eq!(
//             &shuffle_fee.denom,
//             &native_denom,
//             ContractError::InvalidDenom {}
//         );
//         params.extension.shuffle_fee = shuffle_fee;
//     }

//     SUDO_PARAMS.save(deps.storage, &params)?;

//     Ok(Response::new().add_attribute("action", "sudo_update_params"))
// }
