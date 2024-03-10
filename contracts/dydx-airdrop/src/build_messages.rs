use cosmwasm_std::{Addr, BankMsg, coins, DepsMut, MessageInfo, StdResult};
use sg_std::{CosmosMsg, NATIVE_DENOM, Response, SubMsg};
use sg_whitelist::helpers::interface::CollectionWhitelistContract;
use sg_whitelist::msg::AddMembersMsg;
use crate::ContractError;
use crate::query::query_collection_whitelist;
use sg_whitelist::msg::ExecuteMsg as CollectionWhitelistExecuteMsg;


pub fn claim_and_whitelist_add(
    deps: &DepsMut,
    info: MessageInfo,
    airdrop_amount: u128,
) -> Result<Response, ContractError> {
    let mut res = Response::new();
    let bank_msg = SubMsg::new(BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: coins(airdrop_amount, NATIVE_DENOM),
    });
    res = res.add_submessage(bank_msg);
    let collection_whitelist = query_collection_whitelist(deps)?;
    let res = res.add_message(add_member_to_collection_whitelist(
        deps,
        info.sender,
        collection_whitelist,
    )?);
    Ok(res)
}

fn add_member_to_collection_whitelist(
    deps: &DepsMut,
    wallet_address: Addr,
    collection_whitelist: String,
) -> StdResult<CosmosMsg> {
    let inner_msg = AddMembersMsg {
        to_add: vec![wallet_address.to_string()],
    };
    let execute_msg = CollectionWhitelistExecuteMsg::AddMembers(inner_msg);
    CollectionWhitelistContract(deps.api.addr_validate(&collection_whitelist)?)
        .call(execute_msg)
}