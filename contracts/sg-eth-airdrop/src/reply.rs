#[cfg(not(feature = "library"))]
use crate::error::ContractError;
use crate::state::CONFIG;
use cosmwasm_std::entry_point;
use cosmwasm_std::{DepsMut, Env, Reply, Response};
use cw_utils::{parse_instantiate_response_data, MsgInstantiateContractResponse, ParseReplyError};

const INIT_WHITELIST_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    if msg.id != INIT_WHITELIST_REPLY_ID {
        return Err(ContractError::InvalidReplyID {});
    }
    let result = msg
        .result
        .into_result()
        .map_err(|_| ContractError::ReplyOnSuccess {})?;
    let data = result
        .msg_responses
        .first()
        .ok_or(ContractError::ReplyOnSuccess {})?
        .value
        .clone();
    let reply = parse_instantiate_response_data(&data);
    match_reply(deps, reply)
}

fn match_reply(
    deps: DepsMut,
    reply: Result<MsgInstantiateContractResponse, ParseReplyError>,
) -> Result<Response, ContractError> {
    match reply {
        Ok(res) => {
            let whitelist_address = &res.contract_address;
            let mut config = CONFIG.load(deps.storage)?;
            config.whitelist_address = Some(whitelist_address.to_string());
            CONFIG.save(deps.storage, &config)?;

            Ok(Response::default()
                .add_attribute("action", "init_whitelist_reply")
                .add_attribute("whitelist_address", whitelist_address))
        }
        Err(_) => Err(ContractError::ReplyOnSuccess {}),
    }
}
