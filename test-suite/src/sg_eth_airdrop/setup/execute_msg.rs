use crate::sg_eth_airdrop::constants::claim_constants::OWNER;
use crate::sg_eth_airdrop::constants::collection_constants::WHITELIST_AMOUNT;
use crate::{
    common_setup::contract_boxes::{contract_eth_airdrop, contract_whitelist_immutable, App},
    sg_eth_airdrop::setup::test_msgs::InstantiateParams,
};
use anyhow::Error as anyhow_error;
use cosmwasm_std::{coins, Addr};
use cw_multi_test::error::Error;
use cw_multi_test::{AppResponse, BankSudo, Executor, SudoMsg};
use eyre::Result;
use sg_eth_airdrop::msg::{ExecuteMsg, InstantiateMsg};

use sg_std::NATIVE_DENOM;

pub fn instantiate_contract(params: InstantiateParams) -> Result<cosmwasm_std::Addr, anyhow_error> {
    let addresses = params.addresses;
    let minter_address = params.minter_address;
    let admin_account = params.admin_account;
    let funds_amount = params.funds_amount;
    let per_address_limit = params.per_address_limit;
    let claim_msg_plaintext = params.claim_msg_plaintext;
    params
        .app
        .sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: admin_account.to_string(),
                amount: coins(params.funds_amount, NATIVE_DENOM),
            }
        }))
        .map_err(|err| println!("{err:?}"))
        .ok();

    let sg_eth_id = params.app.store_code(contract_eth_airdrop());
    let whitelist_code_id = params.app.store_code(contract_whitelist_immutable());
    assert_eq!(sg_eth_id, params.expected_airdrop_contract_id);

    let msg: InstantiateMsg = InstantiateMsg {
        admin: Addr::unchecked(OWNER),
        claim_msg_plaintext,
        airdrop_amount: WHITELIST_AMOUNT,
        addresses,
        whitelist_code_id,
        minter_address,
        per_address_limit,
    };
    params.app.instantiate_contract(
        sg_eth_id,
        Addr::unchecked(admin_account.clone()),
        &msg,
        &coins(funds_amount, NATIVE_DENOM),
        "sg-eg-airdrop",
        Some(Addr::unchecked(admin_account).to_string()),
    )
}

pub fn execute_contract_with_msg(
    msg: ExecuteMsg,
    app: &mut App,
    user: Addr,
    target_address: Addr,
) -> Result<AppResponse, Box<Error>> {
    let result = app.execute_contract(user, target_address, &msg, &[]);
    Ok(result.unwrap())
}

pub fn execute_contract_error_with_msg(
    msg: ExecuteMsg,
    app: &mut App,
    user: Addr,
    target_address: Addr,
) -> String {
    let result = app
        .execute_contract(user, target_address, &msg, &[])
        .unwrap_err();
    result.root_cause().to_string()
}
