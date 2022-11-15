use crate::constants::{GENERIC_WHITELIST_LABEL, INIT_WHITELIST_REPLY_ID, NATIVE_DENOM};
use crate::contract::get_collection_whitelist;
#[cfg(not(feature = "library"))]
use crate::msg::InstantiateMsg;
use crate::responses::get_remove_eligible_eth_response;
use crate::ContractError;
use cosmwasm_std::{coins, Addr, BankMsg};
use cosmwasm_std::{to_binary, DepsMut, Env, MessageInfo, StdResult, WasmMsg};
use sg_std::{CosmosMsg, Response, StargazeMsgWrapper, SubMsg};
use sg_whitelist::interface::CollectionWhitelistContract;
use sg_whitelist::msg::AddMembersMsg;
use sg_whitelist::msg::ExecuteMsg as CollectionWhitelistExecuteMsg;
use whitelist_generic::helpers::WhitelistGenericContract;
use whitelist_generic::msg::ExecuteMsg as WGExecuteMsg;
use whitelist_generic::msg::InstantiateMsg as WGInstantiateMsg;

pub fn build_whitelist_instantiate_msg(
    env: Env,
    msg: InstantiateMsg,
) -> cosmwasm_std::SubMsg<StargazeMsgWrapper> {
    let whitelist_instantiate_msg = WGInstantiateMsg {
        addresses: msg.addresses,
        mint_discount_bps: Some(0),
        per_address_limit: 1,
    };
    let wasm_msg = WasmMsg::Instantiate {
        code_id: msg.whitelist_code_id,
        admin: Some(env.contract.address.to_string()),
        funds: vec![],
        label: GENERIC_WHITELIST_LABEL.to_string(),
        msg: to_binary(&whitelist_instantiate_msg).unwrap(),
    };
    SubMsg::reply_on_success(wasm_msg, INIT_WHITELIST_REPLY_ID)
}

pub fn build_bank_message(info: MessageInfo, airdrop_amount: u128) -> SubMsg {
    SubMsg::new(BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: coins(airdrop_amount, NATIVE_DENOM),
    })
}

pub fn build_add_eth_eligible_msg(
    deps: DepsMut,
    addresses: Vec<String>,
    whitelist_address: String,
) -> StdResult<CosmosMsg> {
    let execute_msg = WGExecuteMsg::AddAddresses { addresses };
    WhitelistGenericContract(deps.api.addr_validate(&whitelist_address)?).call(execute_msg)
}

pub fn build_remove_eth_eligible_msg(
    deps: &DepsMut,
    eth_address: String,
    whitelist_address: String,
) -> StdResult<CosmosMsg> {
    let execute_msg = WGExecuteMsg::RemoveAddresses {
        addresses: vec![eth_address],
    };
    WhitelistGenericContract(deps.api.addr_validate(&whitelist_address)?).call(execute_msg)
}

pub fn build_update_minter_address_msg(
    deps: DepsMut,
    whitelist_address: String,
    minter_address: String,
) -> StdResult<CosmosMsg> {
    let execute_msg = WGExecuteMsg::UpdateMinterContract {
        minter_contract: minter_address,
    };
    WhitelistGenericContract(deps.api.addr_validate(&whitelist_address)?).call(execute_msg)
}

pub fn build_add_member_minter_msg(
    deps: DepsMut,
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
    deps: DepsMut,
    info: MessageInfo,
    eth_address: String,
    airdrop_amount: u128,
) -> Result<Response, ContractError> {
    let mut res = get_remove_eligible_eth_response(&deps, eth_address).unwrap();
    res = res.add_submessage(build_bank_message(info.clone(), airdrop_amount));
    let collection_whitelist = get_collection_whitelist(&deps)?;
    let res = res
        .add_message(build_add_member_minter_msg(deps, info.sender, collection_whitelist).unwrap());
    Ok(res)
}
