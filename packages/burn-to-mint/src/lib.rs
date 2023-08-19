use cosmwasm_schema::cw_serde;
use cosmwasm_std::StdError;
use std::env;

// use crate::error::ContractError;
// use crate::msg::{ConfigResponse, ExecuteMsg, TokenUriMsg};
// use crate::state::{increment_token_index, Config, COLLECTION_ADDRESS, CONFIG, STATUS};
pub mod msg;

use msg::TokenUriMsg;

#[cfg(not(feature = "library"))]
use cosmwasm_std::{from_binary, to_binary, CosmosMsg, DepsMut, Env, MessageInfo, WasmMsg};

use cw721::Cw721ReceiveMsg;
use sg_std::Response;

#[cw_serde]
pub struct BurnToMint();

pub fn execute_burn_to_mint<T>(
    info: MessageInfo,
    env: Env,
    msg: Cw721ReceiveMsg,
    execute_mint_msg: T,
) -> Result<Response, StdError> {
    let mut res = Response::new();
    let burn_msg = cw721::Cw721ExecuteMsg::Burn {
        token_id: msg.token_id,
    };
    let cosmos_burn_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: info.sender.to_string(),
        msg: to_binary(&burn_msg)?,
        funds: vec![],
    });
    res = res.add_message(cosmos_burn_msg);
    let execute_msg = to_binary(&execute_mint_msg)?;
    // let execute_mint_msg = to_binary(&execute_mint_msg)?;
    let cosmos_mint_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&execute_msg)?,
        funds: vec![],
    });
    let res = res.add_message(cosmos_mint_msg);
    Ok(res)
}
