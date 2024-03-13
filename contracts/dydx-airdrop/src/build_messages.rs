use cosmwasm_std::{
    coins, to_json_binary, Addr, BankMsg, Deps, DepsMut, Env, MessageInfo, StdResult, WasmMsg,
};
use sg_std::{CosmosMsg, Response, StargazeMsgWrapper, SubMsg, NATIVE_DENOM};
use whitelist_updatable::msg::ExecuteMsg;
// TODO: Replace with whitelist_updatable_flatrate
use crate::msg::InstantiateMsg;
use crate::query::{query_collection_whitelist, query_mint_count};
use crate::state::{Config, CONFIG};
use crate::ContractError;
use sg_whitelist_flex::helpers::interface::CollectionWhitelistContract;
use sg_whitelist_flex::msg::AddMembersMsg;
use sg_whitelist_flex::msg::{ExecuteMsg as CollectionWhitelistExecuteMsg, Member};
use whitelist_immutable_flex::msg::InstantiateMsg as WIFInstantiateMsg;
use whitelist_updatable::msg::ExecuteMsg::AddAddresses;
pub const IMMUTABLE_WHITELIST_LABEL: &str = "Whitelist Immutable Flex for Airdrop";
pub const INIT_WHITELIST_REPLY_ID: u64 = 1;

pub fn whitelist_instantiate(
    env: Env,
    msg: InstantiateMsg,
) -> Result<cosmwasm_std::SubMsg<StargazeMsgWrapper>, ContractError> {
    let whitelist_instantiate_msg = WIFInstantiateMsg {
        members: msg.members,
        mint_discount_bps: Some(0),
    };
    let wasm_msg = WasmMsg::Instantiate {
        code_id: msg.whitelist_code_id,
        admin: Some(env.contract.address.to_string()),
        funds: vec![],
        label: IMMUTABLE_WHITELIST_LABEL.to_string(),
        msg: to_json_binary(&whitelist_instantiate_msg)?,
    };
    Ok(SubMsg::reply_on_success(wasm_msg, INIT_WHITELIST_REPLY_ID))
}

pub fn state_config(
    deps: Deps,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Config, ContractError> {
    Ok(Config {
        admin: info.sender,
        claim_msg_plaintext: msg.clone().claim_msg_plaintext,
        airdrop_amount: crate::validation::validate_airdrop_amount(msg.airdrop_amount)?,
        whitelist_address: None,
        minter_address: deps.api.addr_validate(msg.minter_address.as_ref())?,
        name_discount_wl_address: deps
            .api
            .addr_validate(msg.name_discount_wl_address.as_ref())?,
        name_collection_address: deps
            .api
            .addr_validate(msg.name_collection_address.as_ref())?,
        airdrop_count_limit: msg.airdrop_count_limit,
    })
}

pub fn claim_reward(info: MessageInfo, airdrop_amount: u128) -> Result<Response, ContractError> {
    let mut res = Response::new();
    let bank_msg = SubMsg::new(BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: coins(airdrop_amount, NATIVE_DENOM),
    });
    res = res.add_submessage(bank_msg);

    Ok(res)
}

pub fn dust_and_whitelist_add(
    deps: &DepsMut,
    info: MessageInfo,
    eth_address: String,
) -> Result<Response, ContractError> {
    let res = add_member_to_whitelists(deps, info.clone(), eth_address)?;
    let res = res.add_message(dust_member_wallet(info.clone())?);
    Ok(res)
}
pub fn add_member_to_whitelists(
    deps: &DepsMut,
    info: MessageInfo,
    eth_address: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let collection_whitelist = query_collection_whitelist(deps)?;
    let mint_count = query_mint_count(deps, eth_address.clone())?;
    let names_discount_whitelist = config.name_discount_wl_address;

    let res = Response::new();
    let res = res.add_message(add_member_to_collection_whitelist(
        deps,
        info.sender.clone(),
        collection_whitelist,
        mint_count,
    )?);
    let res = res.add_message(add_member_to_names_discount_whitelist(
        info.sender.clone(),
        names_discount_whitelist.clone().to_string(),
    )?);
    Ok(res)
}

fn add_member_to_collection_whitelist(
    deps: &DepsMut,
    wallet_address: Addr,
    collection_whitelist: String,
    mint_count: u32,
) -> StdResult<CosmosMsg> {
    let inner_msg = AddMembersMsg {
        to_add: vec![Member {
            address: wallet_address.to_string(),
            mint_count,
        }],
    };
    let execute_msg = CollectionWhitelistExecuteMsg::AddMembers(inner_msg);
    CollectionWhitelistContract(deps.api.addr_validate(&collection_whitelist)?).call(execute_msg)
}

fn add_member_to_names_discount_whitelist(
    wallet_address: Addr,
    name_discount_wl: String,
) -> StdResult<CosmosMsg> {
    let execute_msg: ExecuteMsg = AddAddresses {
        addresses: vec![wallet_address.to_string()],
    };
    let msg = to_json_binary(&execute_msg)?;
    Ok(WasmMsg::Execute {
        contract_addr: name_discount_wl,
        msg,
        funds: vec![],
    }
    .into())
}

pub fn dust_member_wallet(info: MessageInfo) -> StdResult<CosmosMsg> {
    let inner_msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: coins(1000000, NATIVE_DENOM),
    };
    Ok(CosmosMsg::Bank(inner_msg))
}
