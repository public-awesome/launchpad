use crate::common_setup::{
    contract_boxes::{contract_splits, custom_mock_app, App},
    msg::MinterCollectionResponse,
    setup_accounts_and_block::{instantiate_group, setup_accounts, setup_block_time},
    setup_minter::{
        common::minter_params::minter_params_all,
        vending_minter::setup::{configure_minter, vending_minter_code_ids},
    },
};
use cosmwasm_std::{coins, Addr, Coin, Timestamp};
use cw4::Member;
use cw_multi_test::{next_block, Executor};
use sg2::tests::mock_collection_params_1;

use sg_splits::msg::{ExecuteMsg as SplitsExecuteMsg, Group};
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};

const OWNER: &str = "admin0001";
const MEMBER1: &str = "member0001";
const MEMBER2: &str = "member0002";
const MEMBER3: &str = "member0003";

const MINT_PRICE: u128 = 100_000_000;

pub fn member<T: Into<String>>(addr: T, weight: u64) -> Member {
    Member {
        addr: addr.into(),
        weight,
    }
}

#[track_caller]
fn instantiate_splits(app: &mut App, group_addr: Addr) -> Addr {
    let splits_id = app.store_code(contract_splits());
    println!("splits_id: {splits_id}");
    let msg = sg_splits::msg::InstantiateMsg {
        group: Group::Cw4Address(group_addr.to_string()),
        admin: None,
    };
    app.instantiate_contract(splits_id, Addr::unchecked(OWNER), &msg, &[], "splits", None)
        .unwrap()
}

#[track_caller]
fn setup_splits_test_case(app: &mut App, init_funds: Vec<Coin>) -> (Addr, Addr) {
    // 1. Instantiate group contract with members (and OWNER as admin)
    let members = vec![
        member(OWNER, 50),
        member(MEMBER1, 25),
        member(MEMBER2, 20),
        member(MEMBER3, 5),
    ];
    let group_addr = instantiate_group(app, members);
    app.update_block(next_block);

    // 2. Set up Splits backed by this group
    let splits_addr = instantiate_splits(app, group_addr.clone());
    app.update_block(next_block);

    // Bonus: set some funds on the splits contract for future proposals
    if !init_funds.is_empty() {
        app.send_tokens(Addr::unchecked(OWNER), splits_addr.clone(), &init_funds)
            .unwrap();
    }
    (splits_addr, group_addr)
}

#[test]
fn mint_and_split() {
    let mut app = custom_mock_app();

    let (splits_addr, _) = setup_splits_test_case(&mut app, vec![]);
    let (creator, buyer) = setup_accounts(&mut app);
    let num_tokens = 2;
    let start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    let minter_params = minter_params_all(num_tokens, Some(splits_addr.to_string()), None, None);
    let collection_params = mock_collection_params_1(Some(start_time));
    let code_ids = vending_minter_code_ids(&mut app);
    let minter_collection_response: Vec<MinterCollectionResponse> = configure_minter(
        &mut app,
        creator,
        vec![collection_params],
        vec![minter_params],
        code_ids,
    );
    let minter_addr = minter_collection_response[0].minter.clone().unwrap();
    setup_block_time(&mut app, GENESIS_MINT_START_TIME + 1, None);

    let mint_msg = vending_minter::msg::ExecuteMsg::Mint {};
    let res = app.execute_contract(
        buyer,
        minter_addr,
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_ok());

    let dist_msg = SplitsExecuteMsg::Distribute {};
    let res = app.execute_contract(Addr::unchecked(OWNER), splits_addr, &dist_msg, &[]);
    assert!(res.is_ok());

    let amount = app.wrap().query_balance(OWNER, NATIVE_DENOM).unwrap();
    assert_eq!(amount.amount.u128(), 45000000);
    let amount = app.wrap().query_balance(MEMBER1, NATIVE_DENOM).unwrap();
    assert_eq!(amount.amount.u128(), 22500000);
    let amount = app.wrap().query_balance(MEMBER2, NATIVE_DENOM).unwrap();
    assert_eq!(amount.amount.u128(), 18000000);
    let amount = app.wrap().query_balance(MEMBER3, NATIVE_DENOM).unwrap();
    assert_eq!(amount.amount.u128(), 4500000);
}
