use crate::constants::{GENERIC_WHITELIST_LABEL, INIT_WHITELIST_REPLY_ID, NATIVE_DENOM};
#[cfg(not(feature = "library"))]
use crate::msg::InstantiateMsg;
use cosmwasm_std::{coins, BankMsg};
use cosmwasm_std::{to_binary, DepsMut, Env, MessageInfo, StdResult, WasmMsg};
use sg_std::{CosmosMsg, StargazeMsgWrapper, SubMsg};
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
    deps: DepsMut,
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
