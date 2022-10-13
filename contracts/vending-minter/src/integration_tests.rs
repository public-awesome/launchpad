use crate::contract::instantiate;
use crate::msg::{
    ConfigResponse, ExecuteMsg, MintCountResponse, MintPriceResponse, MintableNumTokensResponse,
    QueryMsg, StartTimeResponse,
};
use crate::ContractError;
use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
use cosmwasm_std::{coin, coins, Addr, Empty, Timestamp, Uint128};
use cosmwasm_std::{Api, Coin};
use cw4::Member;
use cw721::{Cw721QueryMsg, OwnerOfResponse, TokensResponse};
use cw721_base::ExecuteMsg as Cw721ExecuteMsg;
use cw_multi_test::{next_block, BankSudo, Contract, ContractWrapper, Executor, SudoMsg};
use sg2::msg::Sg2ExecuteMsg;
use sg2::tests::mock_collection_params;
use sg721_base::msg::{CollectionInfoResponse, QueryMsg as Sg721QueryMsg};
use sg_multi_test::StargazeApp;
use sg_splits::msg::ExecuteMsg as SplitsExecuteMsg;
use sg_std::{StargazeMsgWrapper, GENESIS_MINT_START_TIME, NATIVE_DENOM};
use sg_whitelist::msg::InstantiateMsg as WhitelistInstantiateMsg;
use sg_whitelist::msg::{
    AddMembersMsg, ConfigResponse as WhitelistConfigResponse, ExecuteMsg as WhitelistExecuteMsg,
    QueryMsg as WhitelistQueryMsg,
};
use vending_factory::msg::{VendingMinterCreateMsg, VendingMinterInitMsgExtension};
use vending_factory::state::{ParamsExtension, VendingMinterParams};

const CREATION_FEE: u128 = 5_000_000_000;
const INITIAL_BALANCE: u128 = 2_000_000_000;

const MINT_PRICE: u128 = 100_000_000;
const MINT_FEE: u128 = 10_000_000;
const WHITELIST_AMOUNT: u128 = 66_000_000;
const WL_PER_ADDRESS_LIMIT: u32 = 1;
const ADMIN_MINT_PRICE: u128 = 0;
const MAX_TOKEN_LIMIT: u32 = 10000;

pub const MIN_MINT_PRICE: u128 = 50_000_000;
pub const AIRDROP_MINT_PRICE: u128 = 0;
pub const MINT_FEE_BPS: u64 = 1_000; // 10%
pub const AIRDROP_MINT_FEE_BPS: u64 = 10_000; // 100%
pub const SHUFFLE_FEE: u128 = 500_000_000;
pub const MAX_PER_ADDRESS_LIMIT: u32 = 50;

fn custom_mock_app() -> StargazeApp {
    StargazeApp::default()
}

pub fn contract_factory() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        vending_factory::contract::execute,
        vending_factory::contract::instantiate,
        vending_factory::contract::query,
    );
    Box::new(contract)
}

pub fn contract_whitelist() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        sg_whitelist::contract::execute,
        sg_whitelist::contract::instantiate,
        sg_whitelist::contract::query,
    );
    Box::new(contract)
}

pub fn contract_minter() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    )
    .with_reply(crate::contract::reply);
    Box::new(contract)
}

pub fn contract_sg721() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        sg721_base::entry::execute,
        sg721_base::entry::instantiate,
        sg721_base::entry::query,
    );
    Box::new(contract)
}

pub fn contract_splits() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new_with_empty(
        sg_splits::contract::execute,
        sg_splits::contract::instantiate,
        sg_splits::contract::query,
    );
    Box::new(contract)
}

pub fn contract_group() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new_with_empty(
        cw4_group::contract::execute,
        cw4_group::contract::instantiate,
        cw4_group::contract::query,
    );
    Box::new(contract)
}

fn setup_whitelist_contract(router: &mut StargazeApp, creator: &Addr) -> Addr {
    let whitelist_code_id = router.store_code(contract_whitelist());

    let msg = WhitelistInstantiateMsg {
        members: vec![],
        start_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME + 100),
        end_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10000000),
        mint_price: coin(WHITELIST_AMOUNT, NATIVE_DENOM),
        per_address_limit: WL_PER_ADDRESS_LIMIT,
        member_limit: 1000,
    };
    router
        .instantiate_contract(
            whitelist_code_id,
            creator.clone(),
            &msg,
            &[coin(100_000_000, NATIVE_DENOM)],
            "whitelist",
            None,
        )
        .unwrap()
}

pub fn mock_params() -> VendingMinterParams {
    VendingMinterParams {
        code_id: 1,
        creation_fee: coin(CREATION_FEE, NATIVE_DENOM),
        min_mint_price: coin(MIN_MINT_PRICE, NATIVE_DENOM),
        mint_fee_bps: MINT_FEE_BPS,
        max_trading_offset_secs: 60 * 60 * 24 * 7,
        extension: ParamsExtension {
            max_token_limit: MAX_TOKEN_LIMIT,
            max_per_address_limit: MAX_PER_ADDRESS_LIMIT,
            airdrop_mint_price: coin(AIRDROP_MINT_PRICE, NATIVE_DENOM),
            airdrop_mint_fee_bps: AIRDROP_MINT_FEE_BPS,
            shuffle_fee: coin(SHUFFLE_FEE, NATIVE_DENOM),
        },
    }
}

pub fn mock_init_extension(splits_addr: Option<String>) -> VendingMinterInitMsgExtension {
    VendingMinterInitMsgExtension {
        base_token_uri: "ipfs://aldkfjads".to_string(),
        payment_address: splits_addr,
        start_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME),
        num_tokens: 100,
        mint_price: coin(MIN_MINT_PRICE, NATIVE_DENOM),
        per_address_limit: 5,
        whitelist: None,
    }
}

pub fn mock_create_minter(splits_addr: Option<String>) -> VendingMinterCreateMsg {
    VendingMinterCreateMsg {
        init_msg: mock_init_extension(splits_addr),
        collection_params: mock_collection_params(),
    }
}

// Upload contract code and instantiate minter contract
fn setup_minter_contract(
    router: &mut StargazeApp,
    creator: &Addr,
    num_tokens: u32,
    splits_addr: Option<String>,
) -> (Addr, ConfigResponse) {
    let minter_code_id = router.store_code(contract_minter());
    println!("minter_code_id: {}", minter_code_id);
    let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);

    let factory_code_id = router.store_code(contract_factory());
    println!("factory_code_id: {}", factory_code_id);

    let mut params = mock_params();
    params.code_id = minter_code_id;

    let factory_addr = router
        .instantiate_contract(
            factory_code_id,
            creator.clone(),
            &vending_factory::msg::InstantiateMsg { params },
            &[],
            "factory",
            None,
        )
        .unwrap();

    let sg721_code_id = router.store_code(contract_sg721());
    println!("sg721_code_id: {}", sg721_code_id);

    let mut msg = mock_create_minter(splits_addr);
    msg.init_msg.mint_price = coin(MINT_PRICE, NATIVE_DENOM);
    msg.init_msg.num_tokens = num_tokens;
    msg.collection_params.code_id = sg721_code_id;
    msg.collection_params.info.creator = creator.to_string();

    let msg = Sg2ExecuteMsg::CreateMinter(msg);

    let res = router.execute_contract(creator.clone(), factory_addr, &msg, &creation_fee);
    assert!(res.is_ok());

    // could get the minter address from the response above, but we know its contract1
    let minter_addr = Addr::unchecked("contract1");

    let config: ConfigResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &QueryMsg::Config {})
        .unwrap();

    (minter_addr, config)
}

fn setup_minter_contract_with_splits(
    router: &mut StargazeApp,
    creator: &Addr,
    num_tokens: u32,
    splits_addr: Option<String>,
) -> (Addr, ConfigResponse) {
    let minter_code_id = router.store_code(contract_minter());
    println!("minter_code_id: {}", minter_code_id);
    let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);

    let factory_code_id = router.store_code(contract_factory());
    println!("factory_code_id: {}", factory_code_id);

    let mut params = mock_params();
    params.code_id = minter_code_id;

    let factory_addr = router
        .instantiate_contract(
            factory_code_id,
            creator.clone(),
            &vending_factory::msg::InstantiateMsg { params },
            &[],
            "factory",
            None,
        )
        .unwrap();

    let sg721_code_id = router.store_code(contract_sg721());
    println!("sg721_code_id: {}", sg721_code_id);

    let mut msg = mock_create_minter(splits_addr);
    msg.init_msg.mint_price = coin(MINT_PRICE, NATIVE_DENOM);
    msg.init_msg.num_tokens = num_tokens;
    msg.collection_params.code_id = sg721_code_id;
    msg.collection_params.info.creator = creator.to_string();

    let msg = Sg2ExecuteMsg::CreateMinter(msg);

    let res = router.execute_contract(creator.clone(), factory_addr, &msg, &creation_fee);
    assert!(res.is_ok());

    // 1 = group, 2 = splits, 3 = minter, 4 = factory, 5 = sg721
    let minter_addr = Addr::unchecked("contract3");

    let config: ConfigResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &QueryMsg::Config {})
        .unwrap();

    (minter_addr, config)
}

fn member<T: Into<String>>(addr: T, weight: u64) -> Member {
    Member {
        addr: addr.into(),
        weight,
    }
}

const OWNER: &str = "admin0001";
const MEMBER1: &str = "member0001";
const MEMBER2: &str = "member0002";
const MEMBER3: &str = "member0003";

// uploads code and returns address of group contract
fn instantiate_group(app: &mut StargazeApp, members: Vec<Member>) -> Addr {
    let group_id = app.store_code(contract_group());
    println!("group_id: {}", group_id);
    let msg = cw4_group::msg::InstantiateMsg {
        admin: Some(OWNER.into()),
        members,
    };
    app.instantiate_contract(group_id, Addr::unchecked(OWNER), &msg, &[], "group", None)
        .unwrap()
}

#[track_caller]
fn instantiate_splits(app: &mut StargazeApp, group: Addr) -> Addr {
    let splits_id = app.store_code(contract_splits());
    println!("splits_id: {}", splits_id);
    let msg = sg_splits::msg::InstantiateMsg {
        group_addr: group.to_string(),
    };
    app.instantiate_contract(splits_id, Addr::unchecked(OWNER), &msg, &[], "splits", None)
        .unwrap()
}

#[track_caller]
fn setup_splits_test_case(app: &mut StargazeApp, init_funds: Vec<Coin>) -> (Addr, Addr) {
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

// Add a creator account with initial balances
fn setup_accounts(router: &mut StargazeApp) -> (Addr, Addr) {
    let buyer = Addr::unchecked("buyer");
    let creator = Addr::unchecked("creator");
    // 3,000 tokens
    let creator_funds = coins(INITIAL_BALANCE + CREATION_FEE, NATIVE_DENOM);
    // 2,000 tokens
    let buyer_funds = coins(INITIAL_BALANCE, NATIVE_DENOM);
    router
        .sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: creator.to_string(),
                amount: creator_funds.clone(),
            }
        }))
        .map_err(|err| println!("{:?}", err))
        .ok();

    router
        .sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: buyer.to_string(),
                amount: buyer_funds.clone(),
            }
        }))
        .map_err(|err| println!("{:?}", err))
        .ok();

    // Check native balances
    let creator_native_balances = router.wrap().query_all_balances(creator.clone()).unwrap();
    assert_eq!(creator_native_balances, creator_funds);

    // Check native balances
    let buyer_native_balances = router.wrap().query_all_balances(buyer.clone()).unwrap();
    assert_eq!(buyer_native_balances, buyer_funds);

    (creator, buyer)
}

// Set blockchain time to after mint by default
fn setup_block_time(router: &mut StargazeApp, nanos: u64, height: Option<u64>) {
    let mut block = router.block_info();
    block.time = Timestamp::from_nanos(nanos);
    if let Some(h) = height {
        block.height = h;
    }
    router.set_block(block);
}

// Deal with zero and non-zero coin amounts for msgs
fn coins_for_msg(msg_coin: Coin) -> Vec<Coin> {
    if msg_coin.amount > Uint128::zero() {
        vec![msg_coin]
    } else {
        vec![]
    }
}

#[test]
fn initialization() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    // Check valid addr
    let addr = "earth1";
    let res = deps.api.addr_validate(addr);
    assert!(res.is_ok());

    // 0 per address limit returns error
    let info = mock_info("creator", &coins(INITIAL_BALANCE, NATIVE_DENOM));
    // let mut msg = minter_init();
    let mut msg = mock_create_minter(None);
    msg.init_msg.num_tokens = 100;
    msg.collection_params.code_id = 1;
    msg.collection_params.info.creator = info.sender.to_string();

    instantiate(deps.as_mut(), mock_env(), info, msg.clone()).unwrap_err();

    // Invalid uri returns error
    let info = mock_info("creator", &coins(INITIAL_BALANCE, NATIVE_DENOM));
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    // Invalid denom returns error
    let wrong_denom = "uosmo";
    let info = mock_info("creator", &coins(INITIAL_BALANCE, NATIVE_DENOM));
    // let mut msg = minter_init();
    let mut msg = mock_create_minter(None);
    // msg.init_msg.mint_price = 100;
    msg.init_msg.mint_price = coin(MINT_PRICE, wrong_denom);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    // Insufficient mint price returns error
    let info = mock_info("creator", &coins(INITIAL_BALANCE, NATIVE_DENOM));
    let mut msg = mock_create_minter(None);
    msg.init_msg.mint_price = coin(1, NATIVE_DENOM);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    // Over max token limit
    let info = mock_info("creator", &coins(INITIAL_BALANCE, NATIVE_DENOM));
    // let mut msg = minter_init();
    let mut msg = mock_create_minter(None);
    msg.init_msg.mint_price = coin(MINT_PRICE, NATIVE_DENOM);
    msg.init_msg.num_tokens = MAX_TOKEN_LIMIT + 1;

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    // Under min token limit
    let info = mock_info("creator", &coins(INITIAL_BALANCE, NATIVE_DENOM));
    // let mut msg = minter_init();
    let mut msg = mock_create_minter(None);
    msg.init_msg.num_tokens = 0;

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();
}

#[test]
fn happy_path() {
    let mut router = custom_mock_app();
    setup_block_time(&mut router, GENESIS_MINT_START_TIME - 1, None);
    let (creator, buyer) = setup_accounts(&mut router);
    let num_tokens = 2;
    let (minter_addr, config) = setup_minter_contract(&mut router, &creator, num_tokens, None);

    // Default start time genesis mint time
    let res: StartTimeResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &QueryMsg::StartTime {})
        .unwrap();
    assert_eq!(
        res.start_time,
        Timestamp::from_nanos(GENESIS_MINT_START_TIME).to_string()
    );

    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 1, None);

    // Fail with incorrect tokens
    let mint_msg = ExecuteMsg::Mint {};
    let err = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(MINT_PRICE + 100, NATIVE_DENOM),
    );
    assert!(err.is_err());

    // Succeeds if funds are sent
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_ok());

    // Balances are correct
    // The creator should get the unit price - mint fee for the mint above
    let creator_balances = router.wrap().query_all_balances(creator.clone()).unwrap();
    assert_eq!(
        creator_balances,
        coins(INITIAL_BALANCE + MINT_PRICE - MINT_FEE, NATIVE_DENOM)
    );
    // The buyer's tokens should reduce by unit price
    let buyer_balances = router.wrap().query_all_balances(buyer.clone()).unwrap();
    assert_eq!(
        buyer_balances,
        coins(INITIAL_BALANCE - MINT_PRICE, NATIVE_DENOM)
    );

    let res: MintCountResponse = router
        .wrap()
        .query_wasm_smart(
            minter_addr.clone(),
            &QueryMsg::MintCount {
                address: buyer.to_string(),
            },
        )
        .unwrap();
    assert_eq!(res.count, 1);
    assert_eq!(res.address, buyer.to_string());

    // Check NFT owned by buyer
    // Random mint token_id 1
    let query_owner_msg = Cw721QueryMsg::OwnerOf {
        token_id: String::from("1"),
        include_expired: None,
    };

    let res: OwnerOfResponse = router
        .wrap()
        .query_wasm_smart(config.sg721_address.clone(), &query_owner_msg)
        .unwrap();
    assert_eq!(res.owner, buyer.to_string());

    // Buyer can't call MintTo
    let mint_to_msg = ExecuteMsg::MintTo {
        recipient: buyer.to_string(),
    };
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_to_msg,
        &coins_for_msg(Coin {
            amount: Uint128::from(ADMIN_MINT_PRICE),
            denom: NATIVE_DENOM.to_string(),
        }),
    );
    assert!(res.is_err());

    // Creator mints an extra NFT for the buyer (who is a friend)
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &mint_to_msg,
        &coins_for_msg(Coin {
            amount: Uint128::from(ADMIN_MINT_PRICE),
            denom: NATIVE_DENOM.to_string(),
        }),
    );
    assert!(res.is_ok());

    // Mint count is not increased if admin mints for the user
    let res: MintCountResponse = router
        .wrap()
        .query_wasm_smart(
            minter_addr.clone(),
            &QueryMsg::MintCount {
                address: buyer.to_string(),
            },
        )
        .unwrap();
    assert_eq!(res.count, 1);
    assert_eq!(res.address, buyer.to_string());

    // Minter contract should have no balance
    let minter_balance = router
        .wrap()
        .query_all_balances(minter_addr.clone())
        .unwrap();
    assert_eq!(0, minter_balance.len());

    // Check that NFT is transferred
    let query_owner_msg = Cw721QueryMsg::OwnerOf {
        token_id: String::from("1"),
        include_expired: None,
    };
    let res: OwnerOfResponse = router
        .wrap()
        .query_wasm_smart(config.sg721_address, &query_owner_msg)
        .unwrap();
    assert_eq!(res.owner, buyer.to_string());

    // Errors if sold out
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins_for_msg(Coin {
            amount: Uint128::from(MINT_PRICE),
            denom: NATIVE_DENOM.to_string(),
        }),
    );
    assert!(res.is_err());

    // Creator can't use MintTo if sold out
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &mint_to_msg,
        &coins_for_msg(Coin {
            amount: Uint128::from(ADMIN_MINT_PRICE),
            denom: NATIVE_DENOM.to_string(),
        }),
    );
    assert!(res.is_err());

    // Can purge after sold out
    let purge_msg = ExecuteMsg::Purge {};
    let res = router.execute_contract(creator, minter_addr.clone(), &purge_msg, &[]);
    assert!(res.is_ok());

    // MintCount should be 0 after purge
    let res: MintCountResponse = router
        .wrap()
        .query_wasm_smart(
            minter_addr,
            &QueryMsg::MintCount {
                address: buyer.to_string(),
            },
        )
        .unwrap();
    assert_eq!(res.count, 0);
}

#[test]

fn invalid_whitelist_instantiate() {
    let mut router = custom_mock_app();
    let (creator, _) = setup_accounts(&mut router);

    let num_tokens = 10;
    let minter_code_id = router.store_code(contract_minter());
    let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);
    let factory_code_id = router.store_code(contract_factory());

    let mut params = mock_params();
    params.code_id = minter_code_id;

    let factory_addr = router
        .instantiate_contract(
            factory_code_id,
            creator.clone(),
            &vending_factory::msg::InstantiateMsg { params },
            &[],
            "factory",
            None,
        )
        .unwrap();

    let sg721_code_id = router.store_code(contract_sg721());

    let mut msg = mock_create_minter(None);
    msg.init_msg.whitelist = Some("invalid address".to_string());
    msg.init_msg.mint_price = coin(MINT_PRICE, NATIVE_DENOM);
    msg.init_msg.num_tokens = num_tokens;
    msg.collection_params.code_id = sg721_code_id;
    msg.collection_params.info.creator = creator.to_string();

    let msg = Sg2ExecuteMsg::CreateMinter(msg);

    let err = router
        .execute_contract(creator, factory_addr, &msg, &creation_fee)
        .unwrap_err();
    assert_eq!(
        err.source().unwrap().source().unwrap().to_string(),
        "Generic error: Querier contract error: cw_multi_test::wasm::ContractData not found"
    );
}

#[test]
fn set_invalid_whitelist() {
    let mut router = custom_mock_app();
    let (creator, _) = setup_accounts(&mut router);
    let num_tokens = 10;
    let (minter_addr, _) = setup_minter_contract(&mut router, &creator, num_tokens, None);
    let whitelist_addr = setup_whitelist_contract(&mut router, &creator);
    const EXPIRATION_TIME: Timestamp = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10_000);

    // Set block to before genesis mint start time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME - 1000, None);

    // Update to a start time after genesis
    let msg = ExecuteMsg::UpdateStartTime(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 1000));
    router
        .execute_contract(creator.clone(), minter_addr.clone(), &msg, &[])
        .unwrap();

    // update wl times
    const WL_START: Timestamp = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 200);

    let wl_msg = WhitelistExecuteMsg::UpdateStartTime(WL_START);
    let res = router.execute_contract(creator.clone(), whitelist_addr.clone(), &wl_msg, &[]);
    assert!(res.is_ok());
    let wl_msg = WhitelistExecuteMsg::UpdateEndTime(EXPIRATION_TIME);
    let res = router.execute_contract(creator.clone(), whitelist_addr.clone(), &wl_msg, &[]);
    assert!(res.is_ok());

    // Set whitelist in minter contract
    let set_whitelist_msg = ExecuteMsg::SetWhitelist {
        whitelist: "invalid".to_string(),
    };
    let err = router
        .execute_contract(
            creator.clone(),
            minter_addr.clone(),
            &set_whitelist_msg,
            &[],
        )
        .unwrap_err();
    assert_eq!(
        err.source().unwrap().source().unwrap().to_string(),
        "Generic error: Querier contract error: cw_multi_test::wasm::ContractData not found"
    );

    // move time to make wl start
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 201, Some(11));

    // check that the new whitelist exists
    let wl_res: WhitelistConfigResponse = router
        .wrap()
        .query_wasm_smart(whitelist_addr.to_string(), &WhitelistQueryMsg::Config {})
        .unwrap();

    assert!(wl_res.is_active);

    // Set whitelist in minter contract
    let set_whitelist_msg = ExecuteMsg::SetWhitelist {
        whitelist: whitelist_addr.to_string(),
    };

    let err = router
        .execute_contract(creator.clone(), minter_addr, &set_whitelist_msg, &[])
        .unwrap_err();
    assert_eq!(err.source().unwrap().to_string(), "WhitelistAlreadyStarted");
}
#[test]
fn mint_count_query() {
    let mut router = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut router);
    let num_tokens = 10;
    let (minter_addr, config) = setup_minter_contract(&mut router, &creator, num_tokens, None);
    let sg721_addr = Addr::unchecked(config.sg721_address);
    let whitelist_addr = setup_whitelist_contract(&mut router, &creator);
    const EXPIRATION_TIME: Timestamp = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10_000);

    // Set block to before genesis mint start time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME - 1000, None);

    let wl_msg = WhitelistExecuteMsg::UpdateEndTime(EXPIRATION_TIME);
    let res = router.execute_contract(creator.clone(), whitelist_addr.clone(), &wl_msg, &[]);
    assert!(res.is_ok());

    let wl_msg = WhitelistExecuteMsg::UpdateStartTime(Timestamp::from_nanos(0));
    let res = router.execute_contract(creator.clone(), whitelist_addr.clone(), &wl_msg, &[]);
    assert!(res.is_ok());

    // Set whitelist in minter contract
    let set_whitelist_msg = ExecuteMsg::SetWhitelist {
        whitelist: whitelist_addr.to_string(),
    };
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &set_whitelist_msg,
        &[],
    );
    assert!(res.is_ok());

    // Update per address_limit
    let set_whitelist_msg = ExecuteMsg::UpdatePerAddressLimit {
        per_address_limit: 3,
    };
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &set_whitelist_msg,
        &[],
    );
    assert!(res.is_ok());

    // Add buyer to whitelist
    let inner_msg = AddMembersMsg {
        to_add: vec![buyer.to_string()],
    };
    let wasm_msg = WhitelistExecuteMsg::AddMembers(inner_msg);
    let res = router.execute_contract(creator.clone(), whitelist_addr, &wasm_msg, &[]);
    assert!(res.is_ok());

    setup_block_time(&mut router, GENESIS_MINT_START_TIME, Some(10));

    // Mint succeeds
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(WHITELIST_AMOUNT, NATIVE_DENOM),
    );
    assert!(res.is_ok());

    // Query count
    let res: MintCountResponse = router
        .wrap()
        .query_wasm_smart(
            minter_addr.clone(),
            &QueryMsg::MintCount {
                address: buyer.to_string(),
            },
        )
        .unwrap();
    assert_eq!(res.count, 1);
    assert_eq!(res.address, buyer.to_string());

    // Mint fails, over whitelist per address limit
    let mint_msg = ExecuteMsg::Mint {};
    let err = router
        .execute_contract(
            buyer.clone(),
            minter_addr.clone(),
            &mint_msg,
            &coins(WHITELIST_AMOUNT, NATIVE_DENOM),
        )
        .unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        ContractError::MaxPerAddressLimitExceeded {}.to_string()
    );

    // Set time after wl ends
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 20_000, Some(11));

    // Public mint succeeds
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_ok());

    // Query count
    let res: MintCountResponse = router
        .wrap()
        .query_wasm_smart(
            minter_addr.clone(),
            &QueryMsg::MintCount {
                address: buyer.to_string(),
            },
        )
        .unwrap();
    assert_eq!(res.count, 2);
    assert_eq!(res.address, buyer.to_string());

    // get random mint token_id
    let tokens_msg = Cw721QueryMsg::Tokens {
        owner: buyer.to_string(),
        start_after: None,
        limit: None,
    };
    let res: TokensResponse = router
        .wrap()
        .query_wasm_smart(sg721_addr.clone(), &tokens_msg)
        .unwrap();
    let sold_token_id: u32 = res.tokens[1].parse::<u32>().unwrap();
    // Buyer transfers NFT to creator
    // random mint token id: 8
    let transfer_msg: Cw721ExecuteMsg<Empty, Empty> = Cw721ExecuteMsg::TransferNft {
        recipient: creator.to_string(),
        // token_id: "8".to_string(),
        token_id: sold_token_id.to_string(),
    };
    let res = router.execute_contract(
        buyer.clone(),
        sg721_addr,
        &transfer_msg,
        &coins_for_msg(coin(123, NATIVE_DENOM)),
    );
    assert!(res.is_ok());

    // Mint succeeds
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_ok());

    // Query count
    let res: MintCountResponse = router
        .wrap()
        .query_wasm_smart(
            minter_addr.clone(),
            &QueryMsg::MintCount {
                address: buyer.to_string(),
            },
        )
        .unwrap();
    assert_eq!(res.count, 3);
    assert_eq!(res.address, buyer.to_string());

    // Mint fails
    let mint_msg = ExecuteMsg::Mint {};
    let err = router
        .execute_contract(
            buyer.clone(),
            minter_addr.clone(),
            &mint_msg,
            &coins(WHITELIST_AMOUNT, NATIVE_DENOM),
        )
        .unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        ContractError::MaxPerAddressLimitExceeded {}.to_string()
    );

    // Query count
    let res: MintCountResponse = router
        .wrap()
        .query_wasm_smart(
            minter_addr,
            &QueryMsg::MintCount {
                address: buyer.to_string(),
            },
        )
        .unwrap();
    assert_eq!(res.count, 3);
    assert_eq!(res.address, buyer.to_string());
}

#[test]
fn whitelist_already_started() {
    let mut router = custom_mock_app();
    let (creator, _) = setup_accounts(&mut router);
    let num_tokens = 1;
    let (minter_addr, _) = setup_minter_contract(&mut router, &creator, num_tokens, None);
    let whitelist_addr = setup_whitelist_contract(&mut router, &creator);

    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 101, None);

    // set whitelist in minter contract
    let set_whitelist_msg = ExecuteMsg::SetWhitelist {
        whitelist: whitelist_addr.to_string(),
    };
    router
        .execute_contract(
            creator.clone(),
            minter_addr,
            &set_whitelist_msg,
            &coins(MINT_PRICE, NATIVE_DENOM),
        )
        .unwrap_err();
}

#[test]
fn whitelist_can_update_before_start() {
    let mut router = custom_mock_app();
    let (creator, _) = setup_accounts(&mut router);
    let num_tokens = 1;
    let (minter_addr, _) = setup_minter_contract(&mut router, &creator, num_tokens, None);
    let whitelist_addr = setup_whitelist_contract(&mut router, &creator);

    setup_block_time(&mut router, GENESIS_MINT_START_TIME - 1000, None);

    // set whitelist in minter contract
    let set_whitelist_msg = ExecuteMsg::SetWhitelist {
        whitelist: whitelist_addr.to_string(),
    };
    router
        .execute_contract(
            creator.clone(),
            minter_addr.clone(),
            &set_whitelist_msg,
            &[],
        )
        .unwrap();

    // can set twice before starting
    router
        .execute_contract(creator.clone(), minter_addr, &set_whitelist_msg, &[])
        .unwrap();
}

#[test]
fn whitelist_access_len_add_remove_expiration() {
    let mut router = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut router);
    let num_tokens = 1;
    let (minter_addr, config) = setup_minter_contract(&mut router, &creator, num_tokens, None);
    let sg721_addr = config.sg721_address;
    let whitelist_addr = setup_whitelist_contract(&mut router, &creator);
    const AFTER_GENESIS_TIME: Timestamp = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 100);

    // Set to just before genesis mint start time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME - 10, None);

    // Update whitelist_expiration fails if not admin
    let wl_msg = WhitelistExecuteMsg::UpdateEndTime(AFTER_GENESIS_TIME);
    router
        .execute_contract(buyer.clone(), whitelist_addr.clone(), &wl_msg, &[])
        .unwrap_err();

    // Update whitelist_expiration succeeds when from admin
    let wl_msg = WhitelistExecuteMsg::UpdateEndTime(AFTER_GENESIS_TIME);
    let res = router.execute_contract(creator.clone(), whitelist_addr.clone(), &wl_msg, &[]);
    assert!(res.is_ok());

    let wl_msg = WhitelistExecuteMsg::UpdateStartTime(Timestamp::from_nanos(0));
    let res = router.execute_contract(creator.clone(), whitelist_addr.clone(), &wl_msg, &[]);
    assert!(res.is_ok());

    // Set whitelist in minter contract
    let set_whitelist_msg = ExecuteMsg::SetWhitelist {
        whitelist: whitelist_addr.to_string(),
    };
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &set_whitelist_msg,
        &[],
    );
    assert!(res.is_ok());

    // Mint fails, buyer is not on whitelist
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_err());

    // Add buyer to whitelist
    let inner_msg = AddMembersMsg {
        to_add: vec![buyer.to_string()],
    };
    let wasm_msg = WhitelistExecuteMsg::AddMembers(inner_msg);
    let res = router.execute_contract(creator.clone(), whitelist_addr.clone(), &wasm_msg, &[]);
    assert!(res.is_ok());

    // Mint fails, not whitelist price
    let mint_msg = ExecuteMsg::Mint {};
    router
        .execute_contract(
            buyer.clone(),
            minter_addr.clone(),
            &mint_msg,
            &coins(MINT_PRICE, NATIVE_DENOM),
        )
        .unwrap_err();

    setup_block_time(&mut router, GENESIS_MINT_START_TIME, None);

    // Query mint price
    let mint_price_response: MintPriceResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &QueryMsg::MintPrice {})
        .unwrap();

    assert_eq!(
        coin(WHITELIST_AMOUNT, NATIVE_DENOM),
        mint_price_response.whitelist_price.unwrap()
    );
    assert_eq!(
        coin(WHITELIST_AMOUNT, NATIVE_DENOM),
        mint_price_response.current_price
    );
    assert_eq!(
        coin(MINT_PRICE, NATIVE_DENOM),
        mint_price_response.public_price
    );

    // Mint succeeds with whitelist price
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(WHITELIST_AMOUNT, NATIVE_DENOM),
    );
    assert!(res.is_ok());

    // Mint fails, over whitelist per address limit
    let mint_msg = ExecuteMsg::Mint {};
    let err = router
        .execute_contract(
            buyer.clone(),
            minter_addr.clone(),
            &mint_msg,
            &coins(WHITELIST_AMOUNT, NATIVE_DENOM),
        )
        .unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        ContractError::MaxPerAddressLimitExceeded {}.to_string()
    );

    // Muyer is generous and transfers to creator
    let transfer_msg: Cw721ExecuteMsg<Empty, Empty> = Cw721ExecuteMsg::TransferNft {
        recipient: creator.to_string(),
        token_id: "1".to_string(),
    };
    let res = router.execute_contract(
        buyer.clone(),
        Addr::unchecked(sg721_addr),
        &transfer_msg,
        &coins_for_msg(coin(123, NATIVE_DENOM)),
    );
    assert!(res.is_ok());

    // Mint fails, buyer exceeded per address limit
    let mint_msg = ExecuteMsg::Mint {};
    let err = router
        .execute_contract(
            buyer.clone(),
            minter_addr.clone(),
            &mint_msg,
            &coins(WHITELIST_AMOUNT, NATIVE_DENOM),
        )
        .unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        ContractError::MaxPerAddressLimitExceeded {}.to_string()
    );

    // Remove buyer from whitelist
    let inner_msg = AddMembersMsg { to_add: vec![] };
    let wasm_msg = WhitelistExecuteMsg::AddMembers(inner_msg);
    let res = router.execute_contract(creator.clone(), whitelist_addr, &wasm_msg, &[]);
    assert!(res.is_ok());

    // Mint fails
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer,
        minter_addr,
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_err());
}

#[test]
fn before_start_time() {
    let mut router = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut router);
    let num_tokens = 1;
    let (minter_addr, _) = setup_minter_contract(&mut router, &creator, num_tokens, None);

    // Set to before genesis mint start time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME - 10, None);

    // Set start_time fails if not admin
    let start_time_msg = ExecuteMsg::UpdateStartTime(Timestamp::from_nanos(0));
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &start_time_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_err());

    // Buyer can't mint before start_time
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_err());

    // Query start_time, confirm expired
    let start_time_response: StartTimeResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &QueryMsg::StartTime {})
        .unwrap();
    assert_eq!(
        Timestamp::from_nanos(GENESIS_MINT_START_TIME).to_string(),
        start_time_response.start_time
    );

    // Set block forward, after start time. mint succeeds
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 10_000_000, None);

    // Mint succeeds
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer,
        minter_addr,
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_ok());
}

#[test]
fn check_per_address_limit() {
    let mut router = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut router);
    let num_tokens = 2;
    let (minter_addr, _config) = setup_minter_contract(&mut router, &creator, num_tokens, None);

    // Set to genesis mint start time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME, None);

    // Set limit, check unauthorized
    let per_address_limit_msg = ExecuteMsg::UpdatePerAddressLimit {
        per_address_limit: 30,
    };
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &per_address_limit_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_err());

    // Set limit errors, invalid limit == 0
    let per_address_limit_msg = ExecuteMsg::UpdatePerAddressLimit {
        per_address_limit: 0,
    };
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &per_address_limit_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_err());

    // Set limit errors, invalid limit over max
    let per_address_limit_msg = ExecuteMsg::UpdatePerAddressLimit {
        per_address_limit: 100,
    };
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &per_address_limit_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_err());

    // Set limit succeeds, mint fails, over max
    let per_address_limit_msg = ExecuteMsg::UpdatePerAddressLimit {
        per_address_limit: 1,
    };
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &per_address_limit_msg,
        &[],
    );
    assert!(res.is_ok());

    // First mint succeeds
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );

    assert!(res.is_ok());

    // Second mint fails from exceeding per address limit
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer,
        minter_addr,
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_err());
}

#[test]
fn check_dynamic_per_address_limit() {
    let mut router = custom_mock_app();
    setup_block_time(&mut router, GENESIS_MINT_START_TIME - 1, None);
    let (creator, _) = setup_accounts(&mut router);

    // if per address limit > 1%, throw error when instantiating
    // num_tokens: 400, per_address_limit: 5
    let num_tokens = 400;
    let minter_code_id = router.store_code(contract_minter());
    let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);

    let factory_code_id = router.store_code(contract_factory());

    let mut params = mock_params();
    params.code_id = minter_code_id;

    let factory_addr = router
        .instantiate_contract(
            factory_code_id,
            creator.clone(),
            &vending_factory::msg::InstantiateMsg { params },
            &[],
            "factory",
            None,
        )
        .unwrap();

    let sg721_code_id = router.store_code(contract_sg721());

    let mut msg = mock_create_minter(None);
    msg.init_msg.mint_price = coin(MINT_PRICE, NATIVE_DENOM);
    msg.init_msg.num_tokens = num_tokens;
    msg.collection_params.code_id = sg721_code_id;
    msg.collection_params.info.creator = creator.to_string();

    let msg = Sg2ExecuteMsg::CreateMinter(msg);

    let err = router
        .execute_contract(creator.clone(), factory_addr.clone(), &msg, &creation_fee)
        .unwrap_err();

    assert_eq!(
        err.source().unwrap().source().unwrap().to_string(),
        ContractError::InvalidPerAddressLimit {
            max: num_tokens / 100,
            min: 1,
            got: mock_create_minter(None).init_msg.per_address_limit,
        }
        .to_string()
    );

    // should succeed with 1000 tokens and 5 per_address_limit
    let num_tokens = 1000;
    let mut msg = mock_create_minter(None);
    msg.init_msg.mint_price = coin(MINT_PRICE, NATIVE_DENOM);
    msg.init_msg.num_tokens = num_tokens;
    msg.collection_params.code_id = sg721_code_id;
    msg.collection_params.info.creator = creator.to_string();
    let msg = Sg2ExecuteMsg::CreateMinter(msg);
    let res = router.execute_contract(creator.clone(), factory_addr, &msg, &creation_fee);
    assert!(res.is_ok());

    let minter_addr = Addr::unchecked("contract1");

    // if per address limit > 1%, throw error when updating per_address_limit
    let update_msg = ExecuteMsg::UpdatePerAddressLimit {
        per_address_limit: 11,
    };
    let err = router
        .execute_contract(creator, minter_addr, &update_msg, &[])
        .unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        ContractError::InvalidPerAddressLimit {
            max: num_tokens / 100,
            min: 1,
            got: 11,
        }
        .to_string()
    );
}

#[test]
fn mint_for_token_id_addr() {
    let mut router = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut router);
    let num_tokens = 4;
    let (minter_addr, config) = setup_minter_contract(&mut router, &creator, num_tokens, None);

    // Set to genesis mint start time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME, None);

    // Try mint_for, test unauthorized
    let mint_for_msg = ExecuteMsg::MintFor {
        token_id: 1,
        recipient: buyer.to_string(),
    };
    let err = router
        .execute_contract(
            buyer.clone(),
            minter_addr.clone(),
            &mint_for_msg,
            &coins_for_msg(Coin {
                amount: Uint128::from(ADMIN_MINT_PRICE),
                denom: NATIVE_DENOM.to_string(),
            }),
        )
        .unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        ContractError::Unauthorized("Sender is not an admin".to_string()).to_string(),
    );

    // Test token id already sold
    // 1. random mint token_id
    // 2. mint_for same token_id
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_ok());

    // get random mint token_id
    let tokens_msg = Cw721QueryMsg::Tokens {
        owner: buyer.to_string(),
        start_after: None,
        limit: None,
    };
    let res: TokensResponse = router
        .wrap()
        .query_wasm_smart(config.sg721_address.clone(), &tokens_msg)
        .unwrap();
    let sold_token_id: u32 = res.tokens[0].parse::<u32>().unwrap();

    // Minter contract should have a balance
    let minter_balance = router
        .wrap()
        .query_all_balances(minter_addr.clone())
        .unwrap();
    assert_eq!(0, minter_balance.len());

    // Mint fails, invalid token_id
    let token_id: u32 = 0;
    let mint_for_msg = ExecuteMsg::MintFor {
        token_id,
        recipient: buyer.to_string(),
    };
    let err = router
        .execute_contract(
            creator.clone(),
            minter_addr.clone(),
            &mint_for_msg,
            &coins_for_msg(Coin {
                amount: Uint128::from(ADMIN_MINT_PRICE),
                denom: NATIVE_DENOM.to_string(),
            }),
        )
        .unwrap_err();
    assert_eq!(
        ContractError::InvalidTokenId {}.to_string(),
        err.source().unwrap().to_string()
    );

    // Mint fails, token_id already sold
    let mint_for_msg = ExecuteMsg::MintFor {
        token_id: sold_token_id,
        recipient: buyer.to_string(),
    };
    let err = router
        .execute_contract(
            creator.clone(),
            minter_addr.clone(),
            &mint_for_msg,
            &coins_for_msg(Coin {
                amount: Uint128::from(ADMIN_MINT_PRICE),
                denom: NATIVE_DENOM.to_string(),
            }),
        )
        .unwrap_err();
    assert_eq!(
        ContractError::TokenIdAlreadySold {
            token_id: sold_token_id
        }
        .to_string(),
        err.source().unwrap().to_string()
    );

    let mintable_num_tokens_response: MintableNumTokensResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &QueryMsg::MintableNumTokens {})
        .unwrap();
    assert_eq!(mintable_num_tokens_response.count, 3);

    // Mint fails, wrong admin airdrop price
    let err = router
        .execute_contract(
            creator.clone(),
            minter_addr.clone(),
            &mint_for_msg,
            &coins_for_msg(Coin {
                amount: Uint128::from(ADMIN_MINT_PRICE + 1),
                denom: NATIVE_DENOM.to_string(),
            }),
        )
        .unwrap_err();
    assert_eq!(
        ContractError::IncorrectPaymentAmount(
            coin(ADMIN_MINT_PRICE + 1, NATIVE_DENOM.to_string()),
            coin(ADMIN_MINT_PRICE, NATIVE_DENOM.to_string())
        )
        .to_string(),
        err.source().unwrap().to_string()
    );

    // Test mint_for token_id 2 then normal mint
    let token_id = 2;
    let mint_for_msg = ExecuteMsg::MintFor {
        token_id,
        recipient: buyer.to_string(),
    };
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &mint_for_msg,
        &coins_for_msg(Coin {
            amount: Uint128::from(ADMIN_MINT_PRICE),
            denom: NATIVE_DENOM.to_string(),
        }),
    );
    assert!(res.is_ok());

    let res: OwnerOfResponse = router
        .wrap()
        .query_wasm_smart(
            config.sg721_address,
            &Cw721QueryMsg::OwnerOf {
                token_id: 2.to_string(),
                include_expired: None,
            },
        )
        .unwrap();
    assert_eq!(res.owner, buyer.to_string());

    let mintable_num_tokens_response: MintableNumTokensResponse = router
        .wrap()
        .query_wasm_smart(minter_addr, &QueryMsg::MintableNumTokens {})
        .unwrap();
    assert_eq!(mintable_num_tokens_response.count, 2);
}

#[test]
fn test_update_start_time() {
    let mut router = custom_mock_app();
    let (creator, _) = setup_accounts(&mut router);
    let num_tokens = 10;

    let (minter_addr, _) = setup_minter_contract(&mut router, &creator, num_tokens, None);

    // Public mint has started
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 100, None);

    // Update to a start time in the past
    let msg = ExecuteMsg::UpdateStartTime(Timestamp::from_nanos(GENESIS_MINT_START_TIME - 1000));
    let err = router
        .execute_contract(creator, minter_addr, &msg, &[])
        .unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        ContractError::AlreadyStarted {}.to_string(),
    );
}

#[test]
fn test_invalid_start_time() {
    let mut router = custom_mock_app();
    let (creator, _) = setup_accounts(&mut router);

    // Upload contract code
    let sg721_code_id = router.store_code(contract_sg721());
    let minter_code_id = router.store_code(contract_minter());
    let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);

    let factory_code_id = router.store_code(contract_factory());

    let mut params = mock_params();
    params.code_id = minter_code_id;

    let factory_addr = router
        .instantiate_contract(
            factory_code_id,
            creator.clone(),
            &vending_factory::msg::InstantiateMsg { params },
            &[],
            "factory",
            None,
        )
        .unwrap();

    // set time before the start_time above
    setup_block_time(&mut router, GENESIS_MINT_START_TIME - 1000, None);

    // Instantiate sale contract before genesis mint
    // let mut minter_init_msg = minter_init();
    let mut minter_msg = mock_create_minter(None);
    minter_msg.init_msg.num_tokens = 10;
    minter_msg.collection_params.code_id = sg721_code_id;
    minter_msg.init_msg.start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME - 100);
    let msg = Sg2ExecuteMsg::CreateMinter(minter_msg.clone());

    router
        .execute_contract(creator.clone(), factory_addr.clone(), &msg, &creation_fee)
        .unwrap_err();

    // move date after genesis mint
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 1000, None);

    // move start time after genesis but before current time
    minter_msg.init_msg.start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 500);
    let msg = Sg2ExecuteMsg::CreateMinter(minter_msg.clone());
    router
        .execute_contract(creator.clone(), factory_addr.clone(), &msg, &creation_fee)
        .unwrap_err();

    // position block time before the start time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 400, None);
    minter_msg.init_msg.start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 500);
    let msg = Sg2ExecuteMsg::CreateMinter(minter_msg);
    router
        .execute_contract(creator.clone(), factory_addr, &msg, &creation_fee)
        .unwrap();

    let minter_addr = Addr::unchecked("contract1");

    // Update to a start time in the past
    let msg = ExecuteMsg::UpdateStartTime(Timestamp::from_nanos(GENESIS_MINT_START_TIME - 100));
    let res = router.execute_contract(creator.clone(), minter_addr.clone(), &msg, &[]);
    assert!(res.is_err());

    // Update to a time after genesis but before the current block_time (GENESIS+400)
    let msg = ExecuteMsg::UpdateStartTime(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 300));
    let res = router.execute_contract(creator.clone(), minter_addr.clone(), &msg, &[]);
    assert!(res.is_err());

    // Update to a time after genesis and after current blocktime (GENESIS+400)
    let msg = ExecuteMsg::UpdateStartTime(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 450));
    let res = router.execute_contract(creator.clone(), minter_addr.clone(), &msg, &[]);
    assert!(res.is_ok());

    // position block after start time (GENESIS+450);
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 500, None);

    // Update to a time after genesis and after current blocktime (GENESIS+400)
    let msg = ExecuteMsg::UpdateStartTime(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 450));
    let err = router
        .execute_contract(creator, minter_addr, &msg, &[])
        .unwrap_err();
    assert_eq!(err.source().unwrap().to_string(), "AlreadyStarted");
}

#[test]
fn invalid_trading_time_during_init() {
    let num_tokens = 10;
    let mut router = custom_mock_app();
    let (creator, _) = setup_accounts(&mut router);

    let minter_code_id = router.store_code(contract_minter());
    println!("minter_code_id: {}", minter_code_id);
    let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);

    let factory_code_id = router.store_code(contract_factory());
    println!("factory_code_id: {}", factory_code_id);

    // set up minter contract
    let mut params = mock_params();
    params.code_id = minter_code_id;
    let factory_addr = router
        .instantiate_contract(
            factory_code_id,
            creator.clone(),
            &vending_factory::msg::InstantiateMsg {
                params: params.clone(),
            },
            &[],
            "factory",
            None,
        )
        .unwrap();

    let sg721_code_id = router.store_code(contract_sg721());
    println!("sg721_code_id: {}", sg721_code_id);

    let mut msg = mock_create_minter(None);
    msg.init_msg.mint_price = coin(MINT_PRICE, NATIVE_DENOM);
    msg.init_msg.num_tokens = num_tokens;
    msg.collection_params.code_id = sg721_code_id;
    msg.collection_params.info.creator = creator.to_string();
    // make trading time beyond factory max trading start time offset
    msg.collection_params.info.start_trading_time = Some(
        msg.init_msg
            .start_time
            .plus_seconds(params.max_trading_offset_secs + 1),
    );

    let msg = Sg2ExecuteMsg::CreateMinter(msg);

    let err = router
        .execute_contract(creator, factory_addr, &msg, &creation_fee)
        .unwrap_err();
    assert!(err
        .source()
        .unwrap()
        .source()
        .unwrap()
        .to_string()
        .contains("InvalidStartTradingTime"));
}

#[test]
fn update_start_trading_time() {
    let mut router = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut router);
    let num_tokens = 2;
    setup_block_time(&mut router, GENESIS_MINT_START_TIME - 1, None);
    let (minter_addr, config) = setup_minter_contract(&mut router, &creator, num_tokens, None);

    // unauthorized
    let res = router.execute_contract(
        Addr::unchecked(buyer),
        Addr::unchecked(minter_addr.clone()),
        &ExecuteMsg::UpdateStartTradingTime(Some(Timestamp::from_nanos(GENESIS_MINT_START_TIME))),
        &[],
    );
    assert!(res.is_err());

    // invalid start trading time
    let res = router.execute_contract(
        Addr::unchecked(creator.clone()),
        Addr::unchecked(minter_addr.clone()),
        &ExecuteMsg::UpdateStartTradingTime(Some(Timestamp::from_nanos(0))),
        &[],
    );
    assert!(res.is_err());

    // invalid start trading time, over offset
    let params = mock_params();
    let res = router.execute_contract(
        Addr::unchecked(creator.clone()),
        Addr::unchecked(minter_addr.clone()),
        &ExecuteMsg::UpdateStartTradingTime(Some(
            Timestamp::from_nanos(GENESIS_MINT_START_TIME)
                .plus_seconds(params.max_trading_offset_secs + 100),
        )),
        &[],
    );
    assert!(res.is_err());

    // succeeds
    let res = router.execute_contract(
        Addr::unchecked(creator.clone()),
        Addr::unchecked(minter_addr),
        &ExecuteMsg::UpdateStartTradingTime(Some(
            Timestamp::from_nanos(GENESIS_MINT_START_TIME)
                .plus_seconds(params.max_trading_offset_secs),
        )),
        &[],
    );
    assert!(res.is_ok());

    // confirm trading start time
    let res: CollectionInfoResponse = router
        .wrap()
        .query_wasm_smart(config.sg721_address, &Sg721QueryMsg::CollectionInfo {})
        .unwrap();
    assert_eq!(
        res.start_trading_time,
        Some(
            Timestamp::from_nanos(GENESIS_MINT_START_TIME)
                .plus_seconds(params.max_trading_offset_secs)
        )
    );
}

#[test]
fn unhappy_path() {
    let mut router = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut router);
    let num_tokens = 1;
    let (minter_addr, _config) = setup_minter_contract(&mut router, &creator, num_tokens, None);

    // Fails if too little funds are sent
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(1, NATIVE_DENOM),
    );
    assert!(res.is_err());

    // Fails if too many funds are sent
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(11111, NATIVE_DENOM),
    );
    assert!(res.is_err());

    // Fails wrong denom is sent
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(buyer, minter_addr, &mint_msg, &coins(MINT_PRICE, "uatom"));
    assert!(res.is_err());
}

#[test]
fn update_mint_price() {
    let mut router = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut router);
    let num_tokens = 1;
    let (minter_addr, _) = setup_minter_contract(&mut router, &creator, num_tokens, None);

    // Set to before genesis mint start time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME - 10, None);

    // Update mint price higher
    let update_msg = ExecuteMsg::UpdateMintPrice {
        price: MINT_PRICE + 1,
    };
    let res = router.execute_contract(creator.clone(), minter_addr.clone(), &update_msg, &[]);
    assert!(res.is_ok());

    // Update mint price lower than initial price
    let update_msg = ExecuteMsg::UpdateMintPrice {
        price: MINT_PRICE - 2,
    };
    let res = router.execute_contract(creator.clone(), minter_addr.clone(), &update_msg, &[]);
    assert!(res.is_ok());

    // Update mint price higher
    let update_msg = ExecuteMsg::UpdateMintPrice {
        price: MINT_PRICE - 1,
    };
    let res = router.execute_contract(creator.clone(), minter_addr.clone(), &update_msg, &[]);
    assert!(res.is_ok());

    // Set block forward, after start time. mint succeeds
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 10_000_000, None);

    // Update mint price higher after start time, throw error
    let update_msg = ExecuteMsg::UpdateMintPrice { price: MINT_PRICE };
    let err = router
        .execute_contract(creator.clone(), minter_addr.clone(), &update_msg, &[])
        .unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        ContractError::UpdatedMintPriceTooHigh {
            allowed: MINT_PRICE - 1,
            updated: MINT_PRICE
        }
        .to_string()
    );

    // Update mint price lower
    let update_msg = ExecuteMsg::UpdateMintPrice {
        price: MINT_PRICE - 2,
    };
    let res = router.execute_contract(creator.clone(), minter_addr.clone(), &update_msg, &[]);
    assert!(res.is_ok());

    // Mint succeeds
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer,
        minter_addr,
        &mint_msg,
        &coins(MINT_PRICE - 2, NATIVE_DENOM),
    );
    assert!(res.is_ok());
}

#[test]
fn mint_and_split() {
    let mut app = custom_mock_app();

    let (splits_addr, _) = setup_splits_test_case(&mut app, vec![]);

    let (creator, buyer) = setup_accounts(&mut app);
    let num_tokens = 2;
    let (minter_addr, _) = setup_minter_contract_with_splits(
        &mut app,
        &creator,
        num_tokens,
        Some(splits_addr.to_string()),
    );
    setup_block_time(&mut app, GENESIS_MINT_START_TIME + 1, None);

    let mint_msg = ExecuteMsg::Mint {};
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

#[test]
fn burn_remaining() {
    let mut router = custom_mock_app();
    setup_block_time(&mut router, GENESIS_MINT_START_TIME - 1, None);
    let (creator, buyer) = setup_accounts(&mut router);
    let num_tokens = 5000;
    let (minter_addr, _) = setup_minter_contract(&mut router, &creator, num_tokens, None);

    // Default start time genesis mint time
    let res: StartTimeResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &QueryMsg::StartTime {})
        .unwrap();
    assert_eq!(
        res.start_time,
        Timestamp::from_nanos(GENESIS_MINT_START_TIME).to_string()
    );

    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 1, None);

    // Succeeds if funds are sent
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_ok());

    // Balances are correct
    // The creator should get the unit price - mint fee for the mint above
    let creator_balances = router.wrap().query_all_balances(creator.clone()).unwrap();
    assert_eq!(
        creator_balances,
        coins(INITIAL_BALANCE + MINT_PRICE - MINT_FEE, NATIVE_DENOM)
    );
    // The buyer's tokens should reduce by unit price
    let buyer_balances = router.wrap().query_all_balances(buyer.clone()).unwrap();
    assert_eq!(
        buyer_balances,
        coins(INITIAL_BALANCE - MINT_PRICE, NATIVE_DENOM)
    );

    let res: MintCountResponse = router
        .wrap()
        .query_wasm_smart(
            minter_addr.clone(),
            &QueryMsg::MintCount {
                address: buyer.to_string(),
            },
        )
        .unwrap();
    assert_eq!(res.count, 1);
    assert_eq!(res.address, buyer.to_string());

    // Buyer can't call MintTo
    let mint_to_msg = ExecuteMsg::MintTo {
        recipient: buyer.to_string(),
    };
    // Creator mints an extra NFT for the buyer (who is a friend)
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &mint_to_msg,
        &coins_for_msg(Coin {
            amount: Uint128::from(ADMIN_MINT_PRICE),
            denom: NATIVE_DENOM.to_string(),
        }),
    );
    assert!(res.is_ok());

    // Mint count is not increased if admin mints for the user
    let res: MintCountResponse = router
        .wrap()
        .query_wasm_smart(
            minter_addr.clone(),
            &QueryMsg::MintCount {
                address: buyer.to_string(),
            },
        )
        .unwrap();
    assert_eq!(res.count, 1);
    assert_eq!(res.address, buyer.to_string());

    // Minter contract should have no balance
    let minter_balance = router
        .wrap()
        .query_all_balances(minter_addr.clone())
        .unwrap();
    assert_eq!(0, minter_balance.len());

    let burn_msg = ExecuteMsg::BurnRemaining {};
    // Creator burns remaining supply
    let res = router.execute_contract(creator.clone(), minter_addr.clone(), &burn_msg, &[]);
    assert!(res.is_ok());
    let burn_msg = ExecuteMsg::BurnRemaining {};
    //  Creator burns remaining supply again but should return sold out
    let err = router
        .execute_contract(creator.clone(), minter_addr.clone(), &burn_msg, &[])
        .unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        ContractError::SoldOut {}.to_string()
    );

    // Errors if sold out
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer,
        minter_addr.clone(),
        &mint_msg,
        &coins_for_msg(Coin {
            amount: Uint128::from(MINT_PRICE),
            denom: NATIVE_DENOM.to_string(),
        }),
    );
    assert!(res.is_err());

    // Creator can't use MintTo if sold out
    let res = router.execute_contract(
        creator,
        minter_addr,
        &mint_to_msg,
        &coins_for_msg(Coin {
            amount: Uint128::from(ADMIN_MINT_PRICE),
            denom: NATIVE_DENOM.to_string(),
        }),
    );
    assert!(res.is_err());
}
