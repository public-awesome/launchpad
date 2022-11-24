use super::validate_airdrop_amount;
use super::{GENERIC_WHITELIST_LABEL, INIT_WHITELIST_REPLY_ID, NATIVE_DENOM};
use crate::msg::InstantiateMsg;
use crate::query::query_collection_whitelist;
use crate::state::Config;
use crate::ContractError;
use cosmwasm_std::{coins, Addr, BankMsg, Deps};
use cosmwasm_std::{to_binary, DepsMut, Env, MessageInfo, StdResult, WasmMsg};
use sg_std::{CosmosMsg, Response, StargazeMsgWrapper, SubMsg};
use sg_whitelist::interface::CollectionWhitelistContract;
use sg_whitelist::msg::AddMembersMsg;
use sg_whitelist::msg::ExecuteMsg as CollectionWhitelistExecuteMsg;
use whitelist_immutable::msg::InstantiateMsg as WGInstantiateMsg;

pub fn build_whitelist_instantiate_msg(
    env: Env,
    msg: InstantiateMsg,
) -> Result<cosmwasm_std::SubMsg<StargazeMsgWrapper>, ContractError> {
    let whitelist_instantiate_msg = WGInstantiateMsg {
        addresses: msg.addresses,
        mint_discount_bps: Some(0),
        per_address_limit: msg.per_address_limit,
    };
    let wasm_msg = WasmMsg::Instantiate {
        code_id: msg.whitelist_code_id,
        admin: Some(env.contract.address.to_string()),
        funds: vec![],
        label: GENERIC_WHITELIST_LABEL.to_string(),
        msg: to_binary(&whitelist_instantiate_msg)?,
    };
    Ok(SubMsg::reply_on_success(wasm_msg, INIT_WHITELIST_REPLY_ID))
}

pub fn build_bank_message(info: MessageInfo, airdrop_amount: u128) -> SubMsg {
    SubMsg::new(BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: coins(airdrop_amount, NATIVE_DENOM),
    })
}

pub fn build_add_member_minter_msg(
    deps: &DepsMut,
    wallet_address: Addr,
    collection_whitelist: String,
) -> StdResult<CosmosMsg> {
    let inner_msg = AddMembersMsg {
        to_add: vec![wallet_address.to_string()],
    };
    let execute_msg = CollectionWhitelistExecuteMsg::AddMembers(inner_msg);
    CollectionWhitelistContract(deps.api.addr_validate(&collection_whitelist)?).call(execute_msg)
}

pub fn build_messages_for_claim_and_whitelist_add(
    deps: &DepsMut,
    info: MessageInfo,
    airdrop_amount: u128,
) -> Result<Response, ContractError> {
    let mut res = Response::new();

    res = res.add_submessage(build_bank_message(info.clone(), airdrop_amount));
    let collection_whitelist = query_collection_whitelist(deps)?;
    let res = res.add_message(build_add_member_minter_msg(
        deps,
        info.sender,
        collection_whitelist,
    )?);
    Ok(res)
}

pub fn build_config_msg(
    deps: Deps,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Config, ContractError> {
    Ok(Config {
        admin: info.sender,
        // TODO validation for size
        claim_msg_plaintext: msg.clone().claim_msg_plaintext,
        airdrop_amount: validate_airdrop_amount(msg.airdrop_amount)?,
        whitelist_address: None,
        minter_address: deps.api.addr_validate(msg.minter_address.as_ref())?,
    })
}
