use cosmwasm_std::{coin, coins, Addr, Timestamp};
use cw_multi_test::{BankSudo, Executor, SudoMsg as CWSudoMsg};

use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};

use sg_whitelist::{
    msg::{AddMembersMsg, ExecuteMsg, InstantiateMsg, MembersResponse, QueryMsg, RemoveMembersMsg},
    state::AdminList,
};

use crate::common_setup::contract_boxes::{contract_collection_whitelist, custom_mock_app, App};

const COLLECTION_WHITELIST_ADDR: &str = "contract0";
const ADMIN: &str = "admin";
const SECOND_ADMIN: &str = "second_admin";
const UNIT_AMOUNT: u128 = 0;

const GENESIS_START_TIME: Timestamp = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
const END_TIME: Timestamp = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 1000);

fn instantiate_contract(admin_account: &str, app: &mut App) {
    let admin = Addr::unchecked(admin_account);
    let funds_amount = 100000000;

    let initial_members = vec!["member0".to_string()];
    let msg = InstantiateMsg {
        members: initial_members,
        start_time: GENESIS_START_TIME,
        end_time: END_TIME,
        mint_price: coin(UNIT_AMOUNT, NATIVE_DENOM),
        per_address_limit: 1,
        member_limit: 1000,
        admins: vec![ADMIN.to_string(), SECOND_ADMIN.to_string()],
        admins_mutable: true,
    };
    app.sudo(CWSudoMsg::Bank({
        BankSudo::Mint {
            to_address: admin.to_string(),
            amount: coins(funds_amount, NATIVE_DENOM),
        }
    }))
    .map_err(|err| println!("{err:?}"))
    .ok();

    let collection_id = app.store_code(contract_collection_whitelist());
    let _ = app.instantiate_contract(
        collection_id,
        admin,
        &msg,
        &coins(funds_amount, NATIVE_DENOM),
        "collection_whitelist".to_string(),
        None,
    );
}

fn add_members_with_specified_admin(admin: &str, members: Vec<String>, app: &mut App) {
    let admin_addr = Addr::unchecked(admin);
    let collection_whitelist_contract = Addr::unchecked("contract0");
    let initial_members = vec!["member0".to_string()];

    let query_msg = QueryMsg::Members {
        start_after: None,
        limit: None,
    };
    let expected_result = MembersResponse {
        members: initial_members.clone(),
    };
    let query_result: MembersResponse = app
        .wrap()
        .query_wasm_smart(collection_whitelist_contract.clone(), &query_msg)
        .unwrap();
    assert_eq!(query_result, expected_result);

    let add_msg = AddMembersMsg {
        to_add: members.clone(),
    };
    let msg = ExecuteMsg::AddMembers(add_msg);
    let res = app.execute_contract(admin_addr, collection_whitelist_contract.clone(), &msg, &[]);
    assert_eq!(res.unwrap().events.len(), 2);

    let query_msg = QueryMsg::Members {
        start_after: None,
        limit: None,
    };
    let expected_result = MembersResponse {
        members: [initial_members, members].concat(),
    };
    let query_result: MembersResponse = app
        .wrap()
        .query_wasm_smart(collection_whitelist_contract, &query_msg)
        .unwrap();
    assert_eq!(query_result, expected_result);
}

fn remove_members_with_specified_admin(admin: &str, members: Vec<String>, app: &mut App) {
    let admin_addr = Addr::unchecked(admin);
    let collection_whitelist_contract = Addr::unchecked("contract0");
    let initial_members = vec![
        "member0".to_string(),
        "member1".to_string(),
        "member2".to_string(),
    ];

    let query_msg = QueryMsg::Members {
        start_after: None,
        limit: None,
    };
    let expected_result = MembersResponse {
        members: initial_members,
    };
    let query_result: MembersResponse = app
        .wrap()
        .query_wasm_smart(collection_whitelist_contract.clone(), &query_msg)
        .unwrap();
    assert_eq!(query_result, expected_result);

    let remove_msg = RemoveMembersMsg { to_remove: members };
    let msg = ExecuteMsg::RemoveMembers(remove_msg);
    let res = app.execute_contract(admin_addr, collection_whitelist_contract.clone(), &msg, &[]);
    assert_eq!(res.unwrap().events.len(), 2);

    let query_msg = QueryMsg::Members {
        start_after: None,
        limit: None,
    };
    let expected_result = MembersResponse {
        members: vec!["member0".to_string()],
    };
    let query_result: MembersResponse = app
        .wrap()
        .query_wasm_smart(collection_whitelist_contract, &query_msg)
        .unwrap();
    assert_eq!(query_result, expected_result);
}

fn add_members_blocked(admin: &str, members: Vec<String>, app: &mut App) {
    let admin_addr = Addr::unchecked(admin);
    let collection_whitelist_contract = Addr::unchecked("contract0");
    let initial_members = vec!["member0".to_string()];

    let query_msg = QueryMsg::Members {
        start_after: None,
        limit: None,
    };
    let expected_result = MembersResponse {
        members: initial_members,
    };
    let query_result: MembersResponse = app
        .wrap()
        .query_wasm_smart(collection_whitelist_contract.clone(), &query_msg)
        .unwrap();
    assert_eq!(query_result, expected_result);

    let add_msg = AddMembersMsg { to_add: members };
    let msg = ExecuteMsg::AddMembers(add_msg);
    let res = app.execute_contract(admin_addr, collection_whitelist_contract.clone(), &msg, &[]);
    assert_eq!(res.unwrap_err().root_cause().to_string(), "Unauthorized");

    let query_result: MembersResponse = app
        .wrap()
        .query_wasm_smart(collection_whitelist_contract, &query_msg)
        .unwrap();
    assert_eq!(query_result, expected_result);
}

#[test]
fn test_instantiate() {
    let mut app = custom_mock_app();
    let admin_account = "admin";
    instantiate_contract(admin_account, &mut app);
}

#[test]
fn test_add_admin() {
    let mut app = custom_mock_app();
    instantiate_contract(ADMIN, &mut app);
    let collection_whitelist_addr = Addr::unchecked(COLLECTION_WHITELIST_ADDR);

    let new_admin: &str = "new_admin";
    let update_admins_message = ExecuteMsg::UpdateAdmins {
        admins: vec![ADMIN.to_string(), new_admin.to_string()],
    };
    let _ = app.execute_contract(
        Addr::unchecked(ADMIN),
        Addr::unchecked(collection_whitelist_addr),
        &update_admins_message,
        &[],
    );
    let members = vec!["member1".to_string(), "member2".to_string()];

    add_members_with_specified_admin(new_admin, members.clone(), &mut app);
    remove_members_with_specified_admin(new_admin, members, &mut app);
}

#[test]
fn test_remove_admin() {
    let mut app = custom_mock_app();
    instantiate_contract(ADMIN, &mut app);
    let collection_whitelist_addr = Addr::unchecked(COLLECTION_WHITELIST_ADDR);

    let update_admin_message = ExecuteMsg::UpdateAdmins {
        admins: vec![ADMIN.to_string()],
    };

    let _ = app.execute_contract(
        Addr::unchecked(ADMIN),
        Addr::unchecked(collection_whitelist_addr),
        &update_admin_message,
        &[],
    );

    let members = vec!["member1".to_string()];
    add_members_blocked(SECOND_ADMIN, members, &mut app);
}

#[test]
fn test_query_admin_list() {
    let mut app = custom_mock_app();
    instantiate_contract(ADMIN, &mut app);
    let collection_whitelist_contract = Addr::unchecked(COLLECTION_WHITELIST_ADDR);

    let query_msg = QueryMsg::AdminList {};
    let query_result: AdminList = app
        .wrap()
        .query_wasm_smart(collection_whitelist_contract, &query_msg)
        .unwrap();
    let expected_result = AdminList {
        admins: vec![Addr::unchecked("admin"), Addr::unchecked("second_admin")],
        mutable: true,
    };
    assert_eq!(query_result, expected_result);
}

#[test]
fn test_freeze_admins() {
    let mut app = custom_mock_app();
    instantiate_contract(ADMIN, &mut app);
    let collection_whitelist_addr = Addr::unchecked(COLLECTION_WHITELIST_ADDR);

    let freeze_admins_msg = ExecuteMsg::Freeze {};
    let _ = app.execute_contract(
        Addr::unchecked(ADMIN),
        Addr::unchecked(collection_whitelist_addr.clone()),
        &freeze_admins_msg,
        &[],
    );

    let update_admin_message = ExecuteMsg::UpdateAdmins {
        admins: vec![ADMIN.to_string()],
    };

    let res = app.execute_contract(
        Addr::unchecked(ADMIN),
        Addr::unchecked(collection_whitelist_addr),
        &update_admin_message,
        &[],
    );
    assert_eq!(res.unwrap_err().root_cause().to_string(), "Unauthorized");
}
