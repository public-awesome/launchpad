use cosmwasm_std::{coin, coins, Addr, Timestamp};
use cw_multi_test::{BankSudo, Executor, IntoAddr, SudoMsg as CWSudoMsg};

use sg_utils::{GENESIS_MINT_START_TIME, NATIVE_DENOM};

use sg_whitelist::{
    msg::{AddMembersMsg, ExecuteMsg, InstantiateMsg, MembersResponse, QueryMsg, RemoveMembersMsg},
    state::AdminList,
};

use crate::common_setup::contract_boxes::{contract_collection_whitelist, custom_mock_app, App};

const UNIT_AMOUNT: u128 = 0;

const GENESIS_START_TIME: Timestamp = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
const END_TIME: Timestamp = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 1000);

// Helper functions for consistent address generation
fn admin_addr() -> Addr {
    "admin".into_addr()
}

fn second_admin_addr() -> Addr {
    "second_admin".into_addr()
}

fn member_addr(seed: &str) -> Addr {
    seed.into_addr()
}

fn instantiate_contract(admin: Addr, app: &mut App) -> Addr {
    let funds_amount = 100000000;

    let member0 = member_addr("member0");
    let msg = InstantiateMsg {
        members: vec![member0.to_string()],
        start_time: GENESIS_START_TIME,
        end_time: END_TIME,
        mint_price: coin(UNIT_AMOUNT, NATIVE_DENOM),
        per_address_limit: 1,
        member_limit: 1000,
        admins: vec![admin_addr().to_string(), second_admin_addr().to_string()],
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
    app.instantiate_contract(
        collection_id,
        admin,
        &msg,
        &coins(funds_amount, NATIVE_DENOM),
        "collection_whitelist".to_string(),
        None,
    )
    .unwrap()
}

fn add_members_with_specified_admin(
    admin: Addr,
    members: Vec<String>,
    contract_addr: Addr,
    app: &mut App,
) {
    let member0 = member_addr("member0").to_string();
    let initial_members = vec![member0];

    let query_msg = QueryMsg::Members {
        start_after: None,
        limit: None,
    };
    let expected_result = MembersResponse {
        members: initial_members.clone(),
    };
    let query_result: MembersResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &query_msg)
        .unwrap();
    assert_eq!(query_result, expected_result);

    let add_msg = AddMembersMsg {
        to_add: members.clone(),
    };
    let msg = ExecuteMsg::AddMembers(add_msg);
    let res = app.execute_contract(admin, contract_addr.clone(), &msg, &[]);
    assert_eq!(res.unwrap().events.len(), 2);

    let query_msg = QueryMsg::Members {
        start_after: None,
        limit: None,
    };
    let mut expected_members = [initial_members, members].concat();
    expected_members.sort();
    let mut actual_members = app
        .wrap()
        .query_wasm_smart::<MembersResponse>(contract_addr, &query_msg)
        .unwrap()
        .members;
    actual_members.sort();
    assert_eq!(actual_members, expected_members);
}

fn remove_members_with_specified_admin(
    admin: Addr,
    members: Vec<String>,
    contract_addr: Addr,
    app: &mut App,
) {
    let mut initial_members = vec![
        member_addr("member0").to_string(),
        member_addr("member1").to_string(),
        member_addr("member2").to_string(),
    ];
    initial_members.sort();

    let query_msg = QueryMsg::Members {
        start_after: None,
        limit: None,
    };
    let mut actual_members = app
        .wrap()
        .query_wasm_smart::<MembersResponse>(contract_addr.clone(), &query_msg)
        .unwrap()
        .members;
    actual_members.sort();
    assert_eq!(actual_members, initial_members);

    let remove_msg = RemoveMembersMsg { to_remove: members };
    let msg = ExecuteMsg::RemoveMembers(remove_msg);
    let res = app.execute_contract(admin, contract_addr.clone(), &msg, &[]);
    assert_eq!(res.unwrap().events.len(), 2);

    let query_msg = QueryMsg::Members {
        start_after: None,
        limit: None,
    };
    let expected_result = MembersResponse {
        members: vec![member_addr("member0").to_string()],
    };
    let query_result: MembersResponse = app
        .wrap()
        .query_wasm_smart(contract_addr, &query_msg)
        .unwrap();
    assert_eq!(query_result, expected_result);
}

fn add_members_blocked(admin: Addr, members: Vec<String>, contract_addr: Addr, app: &mut App) {
    let initial_members = vec![member_addr("member0").to_string()];

    let query_msg = QueryMsg::Members {
        start_after: None,
        limit: None,
    };
    let expected_result = MembersResponse {
        members: initial_members.clone(),
    };
    let query_result: MembersResponse = app
        .wrap()
        .query_wasm_smart(contract_addr.clone(), &query_msg)
        .unwrap();
    assert_eq!(query_result, expected_result);

    let add_msg = AddMembersMsg { to_add: members };
    let msg = ExecuteMsg::AddMembers(add_msg);
    let res = app.execute_contract(admin, contract_addr.clone(), &msg, &[]);
    assert_eq!(res.unwrap_err().root_cause().to_string(), "Unauthorized");

    let query_result: MembersResponse = app
        .wrap()
        .query_wasm_smart(contract_addr, &query_msg)
        .unwrap();
    assert_eq!(query_result, expected_result);
}

#[test]
fn test_instantiate() {
    let mut app = custom_mock_app();
    let admin = admin_addr();
    instantiate_contract(admin, &mut app);
}

#[test]
fn test_add_admin() {
    let mut app = custom_mock_app();
    let admin = admin_addr();
    let contract_addr = instantiate_contract(admin.clone(), &mut app);

    let new_admin = "new_admin".into_addr();
    let update_admins_message = ExecuteMsg::UpdateAdmins {
        admins: vec![admin.to_string(), new_admin.to_string()],
    };
    let _ = app.execute_contract(admin, contract_addr.clone(), &update_admins_message, &[]);
    let members = vec![
        member_addr("member1").to_string(),
        member_addr("member2").to_string(),
    ];

    add_members_with_specified_admin(
        new_admin.clone(),
        members.clone(),
        contract_addr.clone(),
        &mut app,
    );
    remove_members_with_specified_admin(new_admin, members, contract_addr, &mut app);
}

#[test]
fn test_remove_admin() {
    let mut app = custom_mock_app();
    let admin = admin_addr();
    let contract_addr = instantiate_contract(admin.clone(), &mut app);

    let update_admin_message = ExecuteMsg::UpdateAdmins {
        admins: vec![admin.to_string()],
    };

    let _ = app.execute_contract(admin, contract_addr.clone(), &update_admin_message, &[]);

    let members = vec![member_addr("member1").to_string()];
    add_members_blocked(second_admin_addr(), members, contract_addr, &mut app);
}

#[test]
fn test_query_admin_list() {
    let mut app = custom_mock_app();
    let admin = admin_addr();
    let contract_addr = instantiate_contract(admin, &mut app);

    let query_msg = QueryMsg::AdminList {};
    let query_result: AdminList = app
        .wrap()
        .query_wasm_smart(contract_addr, &query_msg)
        .unwrap();
    let expected_result = AdminList {
        admins: vec![admin_addr(), second_admin_addr()],
        mutable: true,
    };
    assert_eq!(query_result, expected_result);
}

#[test]
fn test_freeze_admins() {
    let mut app = custom_mock_app();
    let admin = admin_addr();
    let contract_addr = instantiate_contract(admin.clone(), &mut app);

    let freeze_admins_msg = ExecuteMsg::Freeze {};
    let _ = app.execute_contract(
        admin.clone(),
        contract_addr.clone(),
        &freeze_admins_msg,
        &[],
    );

    let update_admin_message = ExecuteMsg::UpdateAdmins {
        admins: vec![admin.to_string()],
    };

    let res = app.execute_contract(admin, contract_addr, &update_admin_message, &[]);
    assert_eq!(res.unwrap_err().root_cause().to_string(), "Unauthorized");
}
