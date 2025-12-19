use cosmwasm_std::DepsMut;
use cosmwasm_std::Timestamp;
use sg_utils::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
use sg_whitelist::contract::{
    execute, instantiate, query_config, query_members, MAX_MEMBERS, MAX_PER_ADDRESS_LIMIT,
};
use sg_whitelist::error::ContractError;
use sg_whitelist::msg::{
    AddMembersMsg, ConfigResponse, ExecuteMsg, InstantiateMsg, RemoveMembersMsg,
};

use cosmwasm_std::{
    coin,
    testing::{mock_dependencies, mock_env, mock_info, MockApi},
};

const UNIT_AMOUNT: u128 = 100_000_000;

const GENESIS_START_TIME: Timestamp = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
const END_TIME: Timestamp = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 1000);

// Helper to make valid bech32 addresses using MockApi
fn make_addr(seed: &str) -> String {
    MockApi::default().addr_make(seed).to_string()
}

fn setup_contract(deps: DepsMut) {
    let admin = make_addr("admin");
    let second_admin = make_addr("second_admin");
    let member = make_addr("member1");

    let msg = InstantiateMsg {
        members: vec![member],
        start_time: GENESIS_START_TIME,
        end_time: END_TIME,
        mint_price: coin(UNIT_AMOUNT, NATIVE_DENOM),
        per_address_limit: 1,
        member_limit: 1000,
        admins: vec![admin.clone(), second_admin],
        admins_mutable: true,
    };
    let info = mock_info(&admin, &[coin(100_000_000, NATIVE_DENOM)]);
    let res = instantiate(deps, mock_env(), info, msg).unwrap();
    assert_eq!(1, res.messages.len());
}

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());
}

#[test]
fn not_ugaze_denom() {
    let mut deps = mock_dependencies();
    let admin = make_addr("admin");
    let member = make_addr("member1");

    let msg = InstantiateMsg {
        members: vec![member],
        start_time: END_TIME,
        end_time: END_TIME,
        mint_price: coin(UNIT_AMOUNT, "not_ugaze"),
        per_address_limit: 1,
        member_limit: 1000,
        admins: vec![admin.clone()],
        admins_mutable: true,
    };
    let info = mock_info(&admin, &[coin(100_000_000, NATIVE_DENOM)]);
    let res = instantiate(deps.as_mut(), mock_env(), info, msg);
    assert!(res.is_ok());
}

#[test]
fn improper_initialization_invalid_creation_fee() {
    let mut deps = mock_dependencies();
    let admin = make_addr("admin");
    let member = make_addr("member1");

    let msg = InstantiateMsg {
        members: vec![member],
        start_time: END_TIME,
        end_time: END_TIME,
        mint_price: coin(UNIT_AMOUNT, "ugaze"),
        per_address_limit: 1,
        member_limit: 3000,
        admins: vec![admin.clone()],
        admins_mutable: true,
    };
    let info = mock_info(&admin, &[coin(100_000_000, NATIVE_DENOM)]);
    let err = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(
        err.to_string(),
        "IncorrectCreationFee 100000000 < 300000000"
    );
}

#[test]
fn improper_initialization_dedup() {
    let mut deps = mock_dependencies();
    let admin = make_addr("admin");
    let member = make_addr("member1");

    let msg = InstantiateMsg {
        members: vec![member.clone(), member.clone(), member],
        start_time: END_TIME,
        end_time: END_TIME,
        mint_price: coin(UNIT_AMOUNT, NATIVE_DENOM),
        per_address_limit: 1,
        member_limit: 1000,
        admins: vec![admin.clone()],
        admins_mutable: true,
    };
    let info = mock_info(&admin, &[coin(100_000_000, NATIVE_DENOM)]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    let res = query_config(deps.as_ref(), mock_env()).unwrap();
    assert_eq!(1, res.num_members);
}

#[test]
fn check_start_time_after_end_time() {
    let mut deps = mock_dependencies();
    let admin = make_addr("admin");
    let member = make_addr("member1");

    let msg = InstantiateMsg {
        members: vec![member],
        start_time: END_TIME,
        end_time: GENESIS_START_TIME,
        mint_price: coin(UNIT_AMOUNT, NATIVE_DENOM),
        per_address_limit: 1,
        member_limit: 1000,
        admins: vec![admin.clone()],
        admins_mutable: true,
    };
    let info = mock_info(&admin, &[coin(100_000_000, NATIVE_DENOM)]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();
}

#[test]
fn update_start_time() {
    let mut deps = mock_dependencies();
    let admin = make_addr("admin");
    setup_contract(deps.as_mut());

    let msg = ExecuteMsg::UpdateStartTime(Timestamp::from_nanos(GENESIS_MINT_START_TIME - 100));
    let info = mock_info(&admin, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(res.attributes.len(), 3);
    let res = query_config(deps.as_ref(), mock_env()).unwrap();
    assert_eq!(res.start_time, GENESIS_START_TIME);
}

#[test]
fn update_end_time() {
    let mut deps = mock_dependencies();
    let admin = make_addr("admin");
    setup_contract(deps.as_mut());

    let msg = ExecuteMsg::UpdateEndTime(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 100));
    let info = mock_info(&admin, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(res.attributes.len(), 3);

    let msg = ExecuteMsg::UpdateEndTime(Timestamp::from_nanos(GENESIS_MINT_START_TIME - 100));
    let info = mock_info(&admin, &[]);
    execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
}

#[test]
fn update_end_time_after() {
    let mut deps = mock_dependencies();
    let admin = make_addr("admin");
    setup_contract(deps.as_mut());

    let msg = ExecuteMsg::UpdateEndTime(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 100));
    let info = mock_info(&admin, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(res.attributes.len(), 3);

    let mut env = mock_env();
    env.block.time = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 1);

    // after time started should not let increase it
    let msg = ExecuteMsg::UpdateEndTime(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 500));
    let info = mock_info(&admin, &[]);
    assert_eq!(
        execute(deps.as_mut(), env.clone(), info, msg)
            .unwrap_err()
            .to_string(),
        "AlreadyStarted"
    );

    // after time started should let decrease the end time
    let msg = ExecuteMsg::UpdateEndTime(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 50));
    let info = mock_info(&admin, &[]);

    assert!(execute(deps.as_mut(), env.clone(), info, msg).is_ok());

    // after time started should not let decrease before start_time
    let msg = ExecuteMsg::UpdateEndTime(Timestamp::from_nanos(GENESIS_MINT_START_TIME - 50));
    let info = mock_info(&admin, &[]);

    assert!(execute(deps.as_mut(), env, info, msg).is_err());
}

#[test]
fn update_members() {
    let mut deps = mock_dependencies();
    let admin = make_addr("admin");
    let new_member = make_addr("new_member");
    setup_contract(deps.as_mut());

    // dedupe addrs
    let add_msg = AddMembersMsg {
        to_add: vec![new_member.clone(), new_member.clone()],
    };
    let msg = ExecuteMsg::AddMembers(add_msg);
    let info = mock_info(&admin, &[]);
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();
    assert_eq!(res.attributes.len(), 4);
    let res = query_members(deps.as_ref(), None, None).unwrap();
    assert_eq!(res.members.len(), 2);

    // adding duplicate members should succeed
    execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    let res = query_members(deps.as_ref(), None, None).unwrap();
    assert_eq!(res.members.len(), 2);

    let remove_msg = RemoveMembersMsg {
        to_remove: vec![new_member],
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
    let admin = make_addr("admin");
    setup_contract(deps.as_mut());

    let mut members = vec![];
    for i in 0..MAX_MEMBERS {
        members.push(make_addr(&format!("member{i}")));
    }

    let inner_msg = AddMembersMsg { to_add: members };
    let msg = ExecuteMsg::AddMembers(inner_msg);
    let info = mock_info(&admin, &[]);
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
    let admin = make_addr("admin");
    setup_contract(deps.as_mut());

    let per_address_limit: u32 = 50;
    let msg = ExecuteMsg::UpdatePerAddressLimit(per_address_limit);
    let info = mock_info(&admin, &[]);
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
    let info = mock_info(&admin, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(res.attributes.len(), 2);
    let wl_config: ConfigResponse = query_config(deps.as_ref(), mock_env()).unwrap();
    assert_eq!(wl_config.per_address_limit, per_address_limit);
}

#[test]
fn query_members_pagination() {
    let mut deps = mock_dependencies();
    let admin = make_addr("admin");

    let mut members = vec![];
    for i in 0..150 {
        members.push(make_addr(&format!("member{i}")));
    }
    let msg = InstantiateMsg {
        members: members.clone(),
        start_time: END_TIME,
        end_time: END_TIME,
        mint_price: coin(UNIT_AMOUNT, NATIVE_DENOM),
        per_address_limit: 1,
        member_limit: 1000,
        admins: vec![admin.clone()],
        admins_mutable: true,
    };
    let info = mock_info(&admin, &[coin(100_000_000, NATIVE_DENOM)]);
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(1, res.messages.len());

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
    let admin = make_addr("admin");
    setup_contract(deps.as_mut());
    let res = query_config(deps.as_ref(), mock_env()).unwrap();
    assert_eq!(1000, res.member_limit);

    // needs upgrade fee
    let msg = ExecuteMsg::IncreaseMemberLimit(1001);
    let info = mock_info(&admin, &[coin(100_000_000, NATIVE_DENOM)]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    assert!(res.is_ok());

    // 0 upgrade fee
    let msg = ExecuteMsg::IncreaseMemberLimit(1002);
    let info = mock_info(&admin, &[coin(0, NATIVE_DENOM)]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    assert!(res.is_ok());

    // 0 upgrade fee, fails when including a fee
    // don't allow updating to the same number of memebers
    let msg = ExecuteMsg::IncreaseMemberLimit(1002);
    let info = mock_info(&admin, &[coin(1, NATIVE_DENOM)]);
    execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    // 0 upgrade fee, fails when including a fee
    let msg = ExecuteMsg::IncreaseMemberLimit(1003);
    let info = mock_info(&admin, &[coin(1, NATIVE_DENOM)]);
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(err.to_string(), "IncorrectCreationFee 1 < 0");

    // 0 upgrade fee
    let msg = ExecuteMsg::IncreaseMemberLimit(1502);
    let info = mock_info(&admin, &[coin(0, NATIVE_DENOM)]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    assert!(res.is_ok());

    // 0 upgrade fee
    let msg = ExecuteMsg::IncreaseMemberLimit(2000);
    let info = mock_info(&admin, &[coin(0, NATIVE_DENOM)]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    assert!(res.is_ok());

    // needs upgrade fee
    let msg = ExecuteMsg::IncreaseMemberLimit(2002);
    let info = mock_info(&admin, &[coin(100_000_000, NATIVE_DENOM)]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    assert!(res.is_ok());

    // needs upgrade fee
    let msg = ExecuteMsg::IncreaseMemberLimit(4002);
    let info = mock_info(&admin, &[coin(200_000_000, NATIVE_DENOM)]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    assert!(res.is_ok());

    // over MAX_MEMBERS, Invalid member limit
    let msg = ExecuteMsg::IncreaseMemberLimit(6000);
    let info = mock_info(&admin, &[coin(400_000_000, NATIVE_DENOM)]);
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(
        err.to_string(),
        "Invalid member limit. min: 4002, max: 5000, got: 6000"
    );
}

#[test]
fn cant_update_members_non_admin() {
    let mut deps = mock_dependencies();
    let not_admin = make_addr("not_admin");
    let new_member = make_addr("new_member");
    setup_contract(deps.as_mut());

    // dedupe addrs
    let add_msg = AddMembersMsg {
        to_add: vec![new_member.clone(), new_member],
    };
    let msg = ExecuteMsg::AddMembers(add_msg);
    let info = mock_info(&not_admin, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);

    assert_eq!(res.unwrap_err().to_string(), "Unauthorized")
}

fn add_members_with_admin_seed(admin_seed: &str) {
    let mut deps = mock_dependencies();
    let admin = make_addr(admin_seed);
    let new_member = make_addr("new_member");
    setup_contract(deps.as_mut());

    // dedupe addrs
    let add_msg = AddMembersMsg {
        to_add: vec![new_member.clone(), new_member.clone()],
    };
    let msg = ExecuteMsg::AddMembers(add_msg);
    let info = mock_info(&admin, &[]);
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();
    assert_eq!(res.attributes.len(), 4);
    let res = query_members(deps.as_ref(), None, None).unwrap();
    assert_eq!(res.members.len(), 2);

    execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    let res = query_members(deps.as_ref(), None, None).unwrap();
    assert_eq!(res.members.len(), 2);

    let remove_msg = RemoveMembersMsg {
        to_remove: vec![new_member],
    };
    let msg = ExecuteMsg::RemoveMembers(remove_msg);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(res.attributes.len(), 2);
    let res = query_members(deps.as_ref(), None, None).unwrap();
    assert_eq!(res.members.len(), 1);
}

#[test]
fn second_admin_can_add_members() {
    add_members_with_admin_seed("second_admin");
}
