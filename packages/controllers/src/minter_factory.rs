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
