use cosmwasm_std::DepsMut;
use cosmwasm_std::Timestamp;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
use sg_whitelist::contract::{
    execute, instantiate, query_config, query_members, MAX_MEMBERS, MAX_PER_ADDRESS_LIMIT,
};
use sg_whitelist::error::ContractError;
use sg_whitelist::msg::{
    AddMembersMsg, ConfigResponse, ExecuteMsg, InstantiateMsg, RemoveMembersMsg,
};

use cosmwasm_std::{
    coin,
    testing::{mock_dependencies, mock_env, mock_info},
};

const ADMIN: &str = "admin";
const NOT_ADMIN: &str = "not_admin";
const SECOND_ADMIN: &str = "second_admin";
const UNIT_AMOUNT: u128 = 100_000_000;

const GENESIS_START_TIME: Timestamp = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
const END_TIME: Timestamp = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 1000);

fn setup_contract(deps: DepsMut) {
    let msg = InstantiateMsg {
        members: vec!["adsfsa".to_string()],
        start_time: GENESIS_START_TIME,
        end_time: END_TIME,
        mint_price: coin(UNIT_AMOUNT, NATIVE_DENOM),
        per_address_limit: 1,
        member_limit: 1000,
        admins: vec![ADMIN.to_string(), SECOND_ADMIN.to_string()],
        admins_mutable: true,
    };
    let info = mock_info(ADMIN, &[coin(100_000_000, "ustars")]);
    let res = instantiate(deps, mock_env(), info, msg).unwrap();
    assert_eq!(2, res.messages.len());
}

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());
}

#[test]
fn not_ustars_denom() {
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {
        members: vec!["adsfsa".to_string()],
        start_time: END_TIME,
        end_time: END_TIME,
        mint_price: coin(UNIT_AMOUNT, "not_ustars"),
        per_address_limit: 1,
        member_limit: 1000,
        admins: vec![ADMIN.to_string()],
        admins_mutable: true,
    };
    let info = mock_info(ADMIN, &[coin(100_000_000, "ustars")]);
    let res = instantiate(deps.as_mut(), mock_env(), info, msg);
    assert!(res.is_ok());
}

#[test]
fn improper_initialization_invalid_creation_fee() {
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {
        members: vec!["adsfsa".to_string()],
        start_time: END_TIME,
        end_time: END_TIME,
        mint_price: coin(UNIT_AMOUNT, "ustars"),
        per_address_limit: 1,
        member_limit: 3000,
        admins: vec![ADMIN.to_string()],
        admins_mutable: true,
    };
    let info = mock_info(ADMIN, &[coin(100_000_000, "ustars")]);
    let err = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(
        err.to_string(),
        "IncorrectCreationFee 100000000 < 300000000"
    );
}

#[test]
fn improper_initialization_dedup() {
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {
        members: vec![
            "adsfsa".to_string(),
            "adsfsa".to_string(),
            "adsfsa".to_string(),
        ],
        start_time: END_TIME,
        end_time: END_TIME,
        mint_price: coin(UNIT_AMOUNT, NATIVE_DENOM),
        per_address_limit: 1,
        member_limit: 1000,
        admins: vec![ADMIN.to_string()],
        admins_mutable: true,
    };
    let info = mock_info(ADMIN, &[coin(100_000_000, "ustars")]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    let res = query_config(deps.as_ref(), mock_env()).unwrap();
    assert_eq!(1, res.num_members);
}

#[test]
fn check_start_time_after_end_time() {
    let msg = InstantiateMsg {
        members: vec!["adsfsa".to_string()],
        start_time: END_TIME,
        end_time: GENESIS_START_TIME,
        mint_price: coin(UNIT_AMOUNT, NATIVE_DENOM),
        per_address_limit: 1,
        member_limit: 1000,
        admins: vec![ADMIN.to_string()],
        admins_mutable: true,
    };
    let info = mock_info(ADMIN, &[coin(100_000_000, "ustars")]);
    let mut deps = mock_dependencies();
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();
}

#[test]
fn update_start_time() {
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());

    let msg = ExecuteMsg::UpdateStartTime(Timestamp::from_nanos(GENESIS_MINT_START_TIME - 100));
    let info = mock_info(ADMIN, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(res.attributes.len(), 3);
    let res = query_config(deps.as_ref(), mock_env()).unwrap();
    assert_eq!(res.start_time, GENESIS_START_TIME);
}

#[test]
fn update_end_time() {
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());

    let msg = ExecuteMsg::UpdateEndTime(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 100));
    let info = mock_info(ADMIN, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(res.attributes.len(), 3);

    let msg = ExecuteMsg::UpdateEndTime(Timestamp::from_nanos(GENESIS_MINT_START_TIME - 100));
    let info = mock_info(ADMIN, &[]);
    execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
}

#[test]
fn update_end_time_after() {
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());

    let msg = ExecuteMsg::UpdateEndTime(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 100));
    let info = mock_info(ADMIN, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(res.attributes.len(), 3);

    let mut env = mock_env();
    env.block.time = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 1);

    // after time started should not let increase it
    let msg = ExecuteMsg::UpdateEndTime(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 500));
    let info = mock_info(ADMIN, &[]);
    assert_eq!(
        execute(deps.as_mut(), env.clone(), info, msg)
            .unwrap_err()
            .to_string(),
        "AlreadyStarted"
    );

    // after time started should let decrease the end time
    let msg = ExecuteMsg::UpdateEndTime(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 50));
    let info = mock_info(ADMIN, &[]);

    assert!(execute(deps.as_mut(), env.clone(), info, msg).is_ok());

    // after time started should not let decrease before start_time
    let msg = ExecuteMsg::UpdateEndTime(Timestamp::from_nanos(GENESIS_MINT_START_TIME - 50));
    let info = mock_info(ADMIN, &[]);

    assert!(execute(deps.as_mut(), env, info, msg).is_err());
}

#[test]
fn update_members() {
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());

    // dedupe addrs
    let add_msg = AddMembersMsg {
        to_add: vec!["adsfsa1".to_string(), "adsfsa1".to_string()],
    };
    let msg = ExecuteMsg::AddMembers(add_msg);
    let info = mock_info(ADMIN, &[]);
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();
    assert_eq!(res.attributes.len(), 4);
    let res = query_members(deps.as_ref(), None, None).unwrap();
    assert_eq!(res.members.len(), 2);

    // adding duplicate members should succeed
    execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    let res = query_members(deps.as_ref(), None, None).unwrap();
    assert_eq!(res.members.len(), 2);

    let remove_msg = RemoveMembersMsg {
        to_remove: vec!["adsfsa1".to_string()],
    };
    let msg = ExecuteMsg::RemoveMembers(remove_msg);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(res.attributes.len(), 2);
    let res = query_members(deps.as_ref(), None, None).unwrap();
    assert_eq!(res.members.len(), 1);
}

#[test]
fn too_many_members_check() {
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());

    let mut members = vec![];
    for i in 0..MAX_MEMBERS {
        members.push(format!("adsfsa{i}"));
    }

    let inner_msg = AddMembersMsg { to_add: members };
    let msg = ExecuteMsg::AddMembers(inner_msg);
    let info = mock_info(ADMIN, &[]);
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(
        ContractError::MembersExceeded {
            expected: 1000,
            actual: 1000
        }
        .to_string(),
        err.to_string()
    );
}

#[test]
fn update_per_address_limit() {
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());

    let per_address_limit: u32 = 50;
    let msg = ExecuteMsg::UpdatePerAddressLimit(per_address_limit);
    let info = mock_info(ADMIN, &[]);
    // let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    // let wl_config: ConfigResponse = query_config(deps.as_ref(), mock_env()).unwrap();
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(
        ContractError::InvalidPerAddressLimit {
            max: MAX_PER_ADDRESS_LIMIT.to_string(),
            got: per_address_limit.to_string(),
        }
        .to_string(),
        err.to_string()
    );

    let per_address_limit: u32 = 2;
    let msg = ExecuteMsg::UpdatePerAddressLimit(per_address_limit);
    let info = mock_info(ADMIN, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(res.attributes.len(), 2);
    let wl_config: ConfigResponse = query_config(deps.as_ref(), mock_env()).unwrap();
    assert_eq!(wl_config.per_address_limit, per_address_limit);
}
#[test]
fn query_members_pagination() {
    let mut deps = mock_dependencies();
    let mut members = vec![];
    for i in 0..150 {
        members.push(format!("stars1{i}"));
    }
    let msg = InstantiateMsg {
        members: members.clone(),
        start_time: END_TIME,
        end_time: END_TIME,
        mint_price: coin(UNIT_AMOUNT, NATIVE_DENOM),
        per_address_limit: 1,
        member_limit: 1000,
        admins: vec![ADMIN.to_string()],
        admins_mutable: true,
    };
    let info = mock_info(ADMIN, &[coin(100_000_000, "ustars")]);
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(2, res.messages.len());

    let mut all_elements: Vec<String> = vec![];

    // enforcing a min
    let res = query_members(deps.as_ref(), None, None).unwrap();
    assert_eq!(res.members.len(), 25);

    // enforcing a max
    let res = query_members(deps.as_ref(), None, Some(125)).unwrap();
    assert_eq!(res.members.len(), 100);

    // first fetch
    let res = query_members(deps.as_ref(), None, Some(50)).unwrap();
    assert_eq!(res.members.len(), 50);
    all_elements.append(&mut res.members.clone());

    // second
    let res = query_members(
        deps.as_ref(),
        Some(res.members[res.members.len() - 1].clone()),
        Some(50),
    )
    .unwrap();
    assert_eq!(res.members.len(), 50);
    all_elements.append(&mut res.members.clone());

    // third
    let res = query_members(
        deps.as_ref(),
        Some(res.members[res.members.len() - 1].clone()),
        Some(50),
    )
    .unwrap();
    all_elements.append(&mut res.members.clone());
    assert_eq!(res.members.len(), 50);

    // check fetched items
    assert_eq!(all_elements.len(), 150);
    members.sort();
    all_elements.sort();
    assert_eq!(members, all_elements);
}

#[test]
fn increase_member_limit() {
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());
    let res = query_config(deps.as_ref(), mock_env()).unwrap();
    assert_eq!(1000, res.member_limit);

    // needs upgrade fee
    let msg = ExecuteMsg::IncreaseMemberLimit(1001);
    let info = mock_info(ADMIN, &[coin(100_000_000, "ustars")]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    assert!(res.is_ok());

    // 0 upgrade fee
    let msg = ExecuteMsg::IncreaseMemberLimit(1002);
    let info = mock_info(ADMIN, &[coin(0, "ustars")]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    assert!(res.is_ok());

    // 0 upgrade fee, fails when including a fee
    // don't allow updating to the same number of memebers
    let msg = ExecuteMsg::IncreaseMemberLimit(1002);
    let info = mock_info(ADMIN, &[coin(1, "ustars")]);
    execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    // 0 upgrade fee, fails when including a fee
    let msg = ExecuteMsg::IncreaseMemberLimit(1003);
    let info = mock_info(ADMIN, &[coin(1, "ustars")]);
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(err.to_string(), "IncorrectCreationFee 1 < 0");

    // 0 upgrade fee
    let msg = ExecuteMsg::IncreaseMemberLimit(1502);
    let info = mock_info(ADMIN, &[coin(0, "ustars")]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    assert!(res.is_ok());

    // 0 upgrade fee
    let msg = ExecuteMsg::IncreaseMemberLimit(2000);
    let info = mock_info(ADMIN, &[coin(0, "ustars")]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    assert!(res.is_ok());

    // needs upgrade fee
    let msg = ExecuteMsg::IncreaseMemberLimit(2002);
    let info = mock_info(ADMIN, &[coin(100_000_000, "ustars")]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    assert!(res.is_ok());

    // needs upgrade fee
    let msg = ExecuteMsg::IncreaseMemberLimit(4002);
    let info = mock_info(ADMIN, &[coin(200_000_000, "ustars")]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    assert!(res.is_ok());

    // over MAX_MEMBERS, Invalid member limit
    let msg = ExecuteMsg::IncreaseMemberLimit(6000);
    let info = mock_info(ADMIN, &[coin(400_000_000, "ustars")]);
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(
        err.to_string(),
        "Invalid member limit. min: 4002, max: 5000, got: 6000"
    );
}

#[test]
fn cant_update_members_non_admin() {
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());

    // dedupe addrs
    let add_msg = AddMembersMsg {
        to_add: vec!["adsfsa1".to_string(), "adsfsa1".to_string()],
    };
    let msg = ExecuteMsg::AddMembers(add_msg);
    let info = mock_info(NOT_ADMIN, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);

    assert_eq!(res.unwrap_err().to_string(), "Unauthorized")
}

fn add_members_with_specified_admin(admin: &str) {
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());

    // dedupe addrs
    let add_msg = AddMembersMsg {
        to_add: vec!["adsfsa1".to_string(), "adsfsa1".to_string()],
    };
    let msg = ExecuteMsg::AddMembers(add_msg);
    let info = mock_info(admin, &[]);
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();
    assert_eq!(res.attributes.len(), 4);
    let res = query_members(deps.as_ref(), None, None).unwrap();
    assert_eq!(res.members.len(), 2);

    execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    let res = query_members(deps.as_ref(), None, None).unwrap();
    assert_eq!(res.members.len(), 2);

    let remove_msg = RemoveMembersMsg {
        to_remove: vec!["adsfsa1".to_string()],
    };
    let msg = ExecuteMsg::RemoveMembers(remove_msg);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(res.attributes.len(), 2);
    let res = query_members(deps.as_ref(), None, None).unwrap();
    assert_eq!(res.members.len(), 1);
}

#[test]
fn second_admin_can_add_members() {
    add_members_with_specified_admin(SECOND_ADMIN);
}
