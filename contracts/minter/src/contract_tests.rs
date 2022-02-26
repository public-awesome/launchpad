use crate::multi::StargazeApp;
use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
use cosmwasm_std::{coin, coins, Addr, Decimal, Timestamp, Uint128};
use cosmwasm_std::{Api, Coin};
use cw721::{Cw721QueryMsg, OwnerOfResponse};
use cw_multi_test::{BankSudo, Contract, ContractWrapper, Executor, SudoMsg};
use cw_utils::Expiration;
use sg721::msg::InstantiateMsg as Sg721InstantiateMsg;
use sg721::state::{Config, RoyaltyInfo};
use sg_std::{StargazeMsgWrapper, GENESIS_MINT_START_TIME, NATIVE_DENOM};
use whitelist::msg::InstantiateMsg as WhitelistInstantiateMsg;
use whitelist::msg::{ExecuteMsg as WhitelistExecuteMsg, UpdateMembersMsg};

use crate::contract::instantiate;
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, MintableNumTokensResponse, QueryMsg,
    StartTimeResponse,
};
use crate::ContractError;

const CREATION_FEE: u128 = 1_000_000_000;
const INITIAL_BALANCE: u128 = 2_000_000_000;

const UNIT_PRICE: u128 = 100_000_000;
const MINT_FEE: u128 = 10_000_000;
const PRICE: u128 = UNIT_PRICE + MINT_FEE; // 110,000,000
const WHITELIST_AMOUNT: u128 = PRICE + 10_000_000; // set higher to trip tests. irl this should be lower than mint price
const MAX_TOKEN_LIMIT: u32 = 10000;
const MINT_FEE_PERCENT: u64 = 10;

fn custom_mock_app() -> StargazeApp {
    StargazeApp::default()
}
pub fn contract_whitelist() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        whitelist::contract::execute,
        whitelist::contract::instantiate,
        whitelist::contract::query,
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
        sg721::contract::execute,
        sg721::contract::instantiate,
        sg721::contract::query,
    );
    Box::new(contract)
}

fn setup_whitelist_contract(
    router: &mut StargazeApp,
    creator: &Addr,
) -> Result<Addr, ContractError> {
    let whitelist_code_id = router.store_code(contract_whitelist());

    let msg = WhitelistInstantiateMsg {
        members: vec![],
        start_time: Expiration::Never {},
        end_time: Expiration::Never {},
        unit_price: coin(WHITELIST_AMOUNT, NATIVE_DENOM),
    };
    let whitelist_addr = router
        .instantiate_contract(
            whitelist_code_id,
            creator.clone(),
            &msg,
            &[coin(100_000_000, NATIVE_DENOM)],
            "whitelist",
            None,
        )
        .unwrap();

    Ok(whitelist_addr)
}

// Upload contract code and instantiate sale contract
fn setup_minter_contract(
    router: &mut StargazeApp,
    creator: &Addr,
    num_tokens: u64,
) -> Result<(Addr, ConfigResponse), ContractError> {
    // Upload contract code
    let sg721_code_id = router.store_code(contract_sg721());
    let minter_code_id = router.store_code(contract_minter());
    let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);

    // Instantiate sale contract
    let msg = InstantiateMsg {
        unit_price: coin(UNIT_PRICE, NATIVE_DENOM),
        num_tokens,
        start_time: None,
        per_address_limit: None,
        batch_mint_limit: None,
        whitelist: None,
        base_token_uri: "ipfs://QmYxw1rURvnbQbBRTfmVaZtxSrkrfsbodNzibgBrVrUrtN".to_string(),
        sg721_code_id,
        sg721_instantiate_msg: Sg721InstantiateMsg {
            name: String::from("TEST"),
            symbol: String::from("TEST"),
            minter: creator.to_string(),
            config: Some(Config {
                contract_uri: Some(String::from("ipfs://url.json")),
                creator: Some(creator.clone()),
                royalties: Some(RoyaltyInfo {
                    payment_address: creator.clone(),
                    share: Decimal::percent(10),
                }),
            }),
        },
    };
    let minter_addr = router
        .instantiate_contract(
            minter_code_id,
            creator.clone(),
            &msg,
            &creation_fee,
            "Minter",
            None,
        )
        .unwrap();

    let config: ConfigResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &QueryMsg::Config {})
        .unwrap();

    Ok((minter_addr, config))
}

// Add a creator account with initial balances
fn setup_accounts(router: &mut StargazeApp) -> Result<(Addr, Addr), ContractError> {
    let buyer = Addr::unchecked("buyer");
    let creator = Addr::unchecked("creator");
    let creator_funds = coins(INITIAL_BALANCE + CREATION_FEE, NATIVE_DENOM);
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

    Ok((creator, buyer))
}

// set blockchain time to after mint by default
fn setup_block_time(router: &mut StargazeApp, nanos: u64) -> Result<Timestamp, ContractError> {
    let mut block = router.block_info();
    block.time = Timestamp::from_nanos(nanos);
    router.set_block(block.clone());
    Ok(block.time)
}

#[test]
fn initialization() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    // Check valid addr
    let addr = "earth1";
    let res = deps.api.addr_validate(&(*addr));
    assert!(res.is_ok());

    // Invalid uri returns error
    let info = mock_info("creator", &coins(INITIAL_BALANCE, NATIVE_DENOM));
    let msg = InstantiateMsg {
        unit_price: coin(UNIT_PRICE, NATIVE_DENOM),
        num_tokens: 100,
        start_time: None,
        per_address_limit: None,
        batch_mint_limit: None,
        whitelist: None,
        base_token_uri: "https://QmYxw1rURvnbQbBRTfmVaZtxSrkrfsbodNzibgBrVrUrtN".to_string(),
        sg721_code_id: 1,
        sg721_instantiate_msg: Sg721InstantiateMsg {
            name: String::from("TEST"),
            symbol: String::from("TEST"),
            minter: info.sender.to_string(),
            config: Some(Config {
                contract_uri: Some(String::from("test")),
                creator: Some(info.sender.clone()),
                royalties: Some(RoyaltyInfo {
                    payment_address: info.sender.clone(),
                    share: Decimal::percent(10),
                }),
            }),
        },
    };
    let res = instantiate(deps.as_mut(), mock_env(), info, msg);
    assert!(res.is_err());

    // invalid denom returns error
    let wrong_denom = "uosmo";
    let info = mock_info("creator", &coins(INITIAL_BALANCE, NATIVE_DENOM));
    let msg = InstantiateMsg {
        unit_price: coin(UNIT_PRICE, wrong_denom),
        num_tokens: 100,
        start_time: None,
        per_address_limit: None,
        batch_mint_limit: None,
        whitelist: None,
        base_token_uri: "ipfs://QmYxw1rURvnbQbBRTfmVaZtxSrkrfsbodNzibgBrVrUrtN".to_string(),
        sg721_code_id: 1,
        sg721_instantiate_msg: Sg721InstantiateMsg {
            name: String::from("TEST"),
            symbol: String::from("TEST"),
            minter: info.sender.to_string(),
            config: Some(Config {
                contract_uri: Some(String::from("test")),
                creator: Some(info.sender.clone()),
                royalties: Some(RoyaltyInfo {
                    payment_address: info.sender.clone(),
                    share: Decimal::percent(10),
                }),
            }),
        },
    };
    let res = instantiate(deps.as_mut(), mock_env(), info, msg);
    assert!(res.is_err());

    // insufficient mint price returns error
    let info = mock_info("creator", &coins(INITIAL_BALANCE, NATIVE_DENOM));
    let msg = InstantiateMsg {
        unit_price: coin(1, NATIVE_DENOM),
        num_tokens: 100,
        start_time: None,
        per_address_limit: None,
        batch_mint_limit: None,
        whitelist: None,
        base_token_uri: "ipfs://QmYxw1rURvnbQbBRTfmVaZtxSrkrfsbodNzibgBrVrUrtN".to_string(),
        sg721_code_id: 1,
        sg721_instantiate_msg: Sg721InstantiateMsg {
            name: String::from("TEST"),
            symbol: String::from("TEST"),
            minter: info.sender.to_string(),
            config: Some(Config {
                contract_uri: Some(String::from("test")),
                creator: Some(info.sender.clone()),
                royalties: Some(RoyaltyInfo {
                    payment_address: info.sender.clone(),
                    share: Decimal::percent(10),
                }),
            }),
        },
    };
    let res = instantiate(deps.as_mut(), mock_env(), info, msg);
    assert!(res.is_err());

    // over max token limit
    let info = mock_info("creator", &coins(INITIAL_BALANCE, NATIVE_DENOM));
    let msg = InstantiateMsg {
        unit_price: coin(UNIT_PRICE, NATIVE_DENOM),
        num_tokens: (MAX_TOKEN_LIMIT + 1).into(),
        start_time: None,
        per_address_limit: None,
        batch_mint_limit: None,
        whitelist: None,
        base_token_uri: "ipfs://QmYxw1rURvnbQbBRTfmVaZtxSrkrfsbodNzibgBrVrUrtN".to_string(),
        sg721_code_id: 1,
        sg721_instantiate_msg: Sg721InstantiateMsg {
            name: String::from("TEST"),
            symbol: String::from("TEST"),
            minter: info.sender.to_string(),
            config: Some(Config {
                contract_uri: Some(String::from("test")),
                creator: Some(info.sender.clone()),
                royalties: Some(RoyaltyInfo {
                    payment_address: info.sender.clone(),
                    share: Decimal::percent(10),
                }),
            }),
        },
    };
    let res = instantiate(deps.as_mut(), mock_env(), info, msg);
    assert!(res.is_err());
}

#[test]
fn happy_path() {
    let mut router = custom_mock_app();
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 1).unwrap();
    let (creator, buyer) = setup_accounts(&mut router).unwrap();
    let num_tokens: u64 = 2;
    let (minter_addr, config) = setup_minter_contract(&mut router, &creator, num_tokens).unwrap();

    // default start time genesis mint time
    let res: StartTimeResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &QueryMsg::StartTime {})
        .unwrap();
    assert_eq!(
        res.start_time,
        "expiration time: ".to_owned()
            + &Timestamp::from_nanos(GENESIS_MINT_START_TIME).to_string()
    );

    // Fail with incorrect tokens
    let mint_msg = ExecuteMsg::Mint {};
    let err = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(PRICE + 100, NATIVE_DENOM),
    );
    assert!(err.is_err());

    // Succeeds if funds are sent
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(PRICE, NATIVE_DENOM),
    );
    assert!(res.is_ok());

    // Balances are correct
    // The creator should get the unit price for the mint above
    let creator_balances = router.wrap().query_all_balances(creator.clone()).unwrap();
    assert_eq!(
        creator_balances,
        coins(INITIAL_BALANCE + UNIT_PRICE, NATIVE_DENOM)
    );
    // The buyer's tokens should reduce by unit price + mint fee
    let buyer_balances = router.wrap().query_all_balances(buyer.clone()).unwrap();
    assert_eq!(buyer_balances, coins(INITIAL_BALANCE - PRICE, NATIVE_DENOM));

    // Check NFT is transferred
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
        recipient: buyer.clone(),
    };
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_to_msg,
        &coins(PRICE, NATIVE_DENOM),
    );
    assert!(res.is_err());

    // Creator mints an extra NFT for the buyer (who is a friend)
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &mint_to_msg,
        &coins(PRICE, NATIVE_DENOM),
    );
    assert!(res.is_ok());

    // minter contract should have no balance
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
        buyer,
        minter_addr.clone(),
        &mint_msg,
        &coins(PRICE, NATIVE_DENOM),
    );
    assert!(res.is_err());

    // Creator can't use MintFor if sold out
    let res = router.execute_contract(
        creator,
        minter_addr,
        &mint_to_msg,
        &coins(PRICE, NATIVE_DENOM),
    );
    assert!(res.is_err());
}

#[test]
fn whitelist_access_len_add_remove_expiration() {
    let mut router = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut router).unwrap();
    let num_tokens: u64 = 1;
    let (minter_addr, _config) = setup_minter_contract(&mut router, &creator, num_tokens).unwrap();
    let whitelist_addr = setup_whitelist_contract(&mut router, &creator).unwrap();
    const EXPIRATION_TIME: Timestamp = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10_000_000);
    // set to genesis mint start time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME).unwrap();

    // update whitelist_expiration fails if not admin
    let wl_msg = WhitelistExecuteMsg::UpdateEndTime(Expiration::AtTime(EXPIRATION_TIME));
    let res = router.execute_contract(
        buyer.clone(),
        whitelist_addr.clone(),
        &wl_msg,
        &coins(PRICE, NATIVE_DENOM),
    );
    assert!(res.is_err());

    let wl_msg = WhitelistExecuteMsg::UpdateEndTime(Expiration::AtTime(EXPIRATION_TIME));
    let res = router.execute_contract(
        creator.clone(),
        whitelist_addr.clone(),
        &wl_msg,
        &coins(PRICE, NATIVE_DENOM),
    );
    assert!(res.is_ok());

    let wl_msg = WhitelistExecuteMsg::UpdateStartTime(Expiration::AtTime(Timestamp::from_nanos(0)));
    let res = router.execute_contract(
        creator.clone(),
        whitelist_addr.clone(),
        &wl_msg,
        &coins(PRICE, NATIVE_DENOM),
    );
    assert!(res.is_ok());

    // set whitelist in minter contract
    let set_whitelist_msg = ExecuteMsg::SetWhitelist {
        whitelist: whitelist_addr.to_string(),
    };
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &set_whitelist_msg,
        &coins(PRICE, NATIVE_DENOM),
    );
    assert!(res.is_ok());

    // mint fails, buyer is not on whitelist
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(PRICE, NATIVE_DENOM),
    );
    assert!(res.is_err());

    let inner_msg = UpdateMembersMsg {
        add: vec![buyer.to_string()],
        remove: vec![],
    };
    let wasm_msg = WhitelistExecuteMsg::UpdateMembers(inner_msg);
    let res = router.execute_contract(
        creator.clone(),
        whitelist_addr.clone(),
        &wasm_msg,
        &coins(PRICE, NATIVE_DENOM),
    );
    assert!(res.is_ok());

    // mint fails, not whitelist price
    let mint_msg = ExecuteMsg::Mint {};
    let mint_fee_percent = Decimal::percent(MINT_FEE_PERCENT);
    let mint_price: Uint128 =
        (Uint128::from(WHITELIST_AMOUNT) * mint_fee_percent) + Uint128::from(WHITELIST_AMOUNT);
    let err = router
        .execute_contract(
            buyer.clone(),
            minter_addr.clone(),
            &mint_msg,
            &coins(PRICE, NATIVE_DENOM),
        )
        .unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        ContractError::IncorrectPaymentAmount(
            Coin {
                amount: PRICE.into(),
                denom: NATIVE_DENOM.into(),
            },
            Coin {
                amount: mint_price,
                denom: NATIVE_DENOM.into(),
            }
        )
        .to_string()
    );

    // mint succeeds
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(mint_price.into(), NATIVE_DENOM),
    );
    assert!(res.is_ok());

    // remove buyer from whitelist
    let inner_msg = UpdateMembersMsg {
        add: vec![],
        remove: vec![buyer.to_string()],
    };
    let wasm_msg = WhitelistExecuteMsg::UpdateMembers(inner_msg);
    let res = router.execute_contract(
        creator.clone(),
        whitelist_addr,
        &wasm_msg,
        &coins(PRICE, NATIVE_DENOM),
    );
    assert!(res.is_ok());

    // mint fails
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(buyer, minter_addr, &mint_msg, &coins(PRICE, NATIVE_DENOM));
    assert!(res.is_err());
}

#[test]
fn before_start_time() {
    let mut router = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut router).unwrap();
    let num_tokens: u64 = 1;
    let (minter_addr, _) = setup_minter_contract(&mut router, &creator, num_tokens).unwrap();
    // set to before genesis mint start time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME - 10).unwrap();

    // set start_time fails if not admin
    let start_time_msg = ExecuteMsg::UpdateStartTime(Expiration::Never {});
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &start_time_msg,
        &coins(PRICE, NATIVE_DENOM),
    );
    assert!(res.is_err());

    // if block before start_time, throw error
    let start_time_msg = ExecuteMsg::UpdateStartTime(Expiration::AtTime(Timestamp::from_nanos(
        GENESIS_MINT_START_TIME,
    )));
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &start_time_msg,
        &coins(PRICE, NATIVE_DENOM),
    );
    assert!(res.is_ok());

    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(PRICE, NATIVE_DENOM),
    );
    assert!(res.is_err());

    // query start_time, confirm expired
    let start_time_response: StartTimeResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &QueryMsg::StartTime {})
        .unwrap();
    assert_eq!(
        "expiration time: ".to_owned()
            + &Timestamp::from_nanos(GENESIS_MINT_START_TIME).to_string(),
        start_time_response.start_time
    );

    // set block forward, after start time. mint succeeds
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 10_000_000).unwrap();

    // mint succeeds
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(buyer, minter_addr, &mint_msg, &coins(PRICE, NATIVE_DENOM));
    assert!(res.is_ok());
}

#[test]
fn check_per_address_limit() {
    let mut router = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut router).unwrap();
    let num_tokens = 2;
    let (minter_addr, _config) = setup_minter_contract(&mut router, &creator, num_tokens).unwrap();
    // set to genesis mint start time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME).unwrap();

    // set limit, check unauthorized
    let per_address_limit_msg = ExecuteMsg::UpdatePerAddressLimit {
        per_address_limit: 30,
    };
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &per_address_limit_msg,
        &coins(PRICE, NATIVE_DENOM),
    );
    assert!(res.is_err());

    // set limit, invalid limit over max
    let per_address_limit_msg = ExecuteMsg::UpdatePerAddressLimit {
        per_address_limit: 100,
    };
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &per_address_limit_msg,
        &coins(PRICE, NATIVE_DENOM),
    );
    assert!(res.is_err());

    // set limit, mint fails, over max
    let per_address_limit_msg = ExecuteMsg::UpdatePerAddressLimit {
        per_address_limit: 1,
    };
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &per_address_limit_msg,
        &coins(PRICE, NATIVE_DENOM),
    );
    assert!(res.is_ok());

    // first mint succeeds
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(PRICE, NATIVE_DENOM),
    );

    assert!(res.is_ok());

    // second mint fails from exceeding per address limit
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(buyer, minter_addr, &mint_msg, &coins(PRICE, NATIVE_DENOM));
    assert!(res.is_err());
}

#[test]
fn batch_mint_limit_access_max_sold_out() {
    let mut router = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut router).unwrap();
    let num_tokens = 4;
    let (minter_addr, _config) = setup_minter_contract(&mut router, &creator, num_tokens).unwrap();
    // set to genesis mint start time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME).unwrap();

    // batch mint limit set to STARTING_BATCH_MINT_LIMIT if no mint provided
    let batch_mint_msg = ExecuteMsg::BatchMint { num_mints: 1 };
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &batch_mint_msg,
        &coins(PRICE, NATIVE_DENOM),
    );
    assert!(res.is_ok());

    // update batch mint limit, test unauthorized
    let update_batch_mint_limit_msg = ExecuteMsg::UpdateBatchMintLimit {
        batch_mint_limit: 1,
    };
    let err = router
        .execute_contract(
            buyer.clone(),
            minter_addr.clone(),
            &update_batch_mint_limit_msg,
            &[],
        )
        .unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        ContractError::Unauthorized("Sender is not an admin".to_string()).to_string()
    );

    // update limit, invalid limit over max
    let update_batch_mint_limit_msg = ExecuteMsg::UpdateBatchMintLimit {
        batch_mint_limit: 100,
    };
    let err = router
        .execute_contract(
            creator.clone(),
            minter_addr.clone(),
            &update_batch_mint_limit_msg,
            &[],
        )
        .unwrap_err();
    assert_eq!(
        ContractError::InvalidBatchMintLimit {
            max: 30.to_string(),
            got: 100.to_string()
        }
        .to_string(),
        err.source().unwrap().to_string()
    );

    // update limit successfully as admin
    let update_batch_mint_limit_msg = ExecuteMsg::UpdateBatchMintLimit {
        batch_mint_limit: 2,
    };
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &update_batch_mint_limit_msg,
        &[],
    );
    assert!(res.is_ok());

    // test over max batch mint limit
    let batch_mint_msg = ExecuteMsg::BatchMint { num_mints: 50 };
    let err = router
        .execute_contract(
            buyer.clone(),
            minter_addr.clone(),
            &batch_mint_msg,
            &coins(PRICE, NATIVE_DENOM),
        )
        .unwrap_err();
    assert_eq!(
        ContractError::MaxBatchMintLimitExceeded {}.to_string(),
        err.source().unwrap().to_string()
    );

    // success
    let num_mints = 2;
    let batch_mint_msg = ExecuteMsg::BatchMint { num_mints };
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &batch_mint_msg,
        &coins(PRICE * (num_mints as u128), NATIVE_DENOM),
    );
    assert!(res.is_ok());

    // test sold out and fails
    let num_mints = 2;
    let batch_mint_msg = ExecuteMsg::BatchMint { num_mints };
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &batch_mint_msg,
        &coins(PRICE * num_mints as u128, NATIVE_DENOM),
    );
    assert!(res.is_err());
    let err = res.unwrap_err();
    if let Some(nested_err) = err.source() {
        assert_eq!(
            nested_err.source().unwrap().to_string(),
            ContractError::SoldOut {}.to_string(),
        );
    };

    // add active whitelist
    let whitelist_addr = setup_whitelist_contract(&mut router, &creator).unwrap();

    // set whitelist in minter contract
    let set_whitelist_msg = ExecuteMsg::SetWhitelist {
        whitelist: whitelist_addr.to_string(),
    };
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &set_whitelist_msg,
        &coins(PRICE, NATIVE_DENOM),
    );
    assert!(res.is_ok());

    // batch mint smaller amount
    let mint_fee_percent = Decimal::percent(MINT_FEE_PERCENT);
    let mint_price: Uint128 =
        (Uint128::from(WHITELIST_AMOUNT) * mint_fee_percent) + Uint128::from(WHITELIST_AMOUNT);
    let batch_mint_msg = ExecuteMsg::BatchMint { num_mints: 1 };
    let res = router.execute_contract(
        buyer,
        minter_addr,
        &batch_mint_msg,
        &coins(mint_price.into(), NATIVE_DENOM),
    );
    assert!(res.is_ok());
}

#[test]
fn mint_for_token_id_addr() {
    let mut router = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut router).unwrap();
    let num_tokens: u64 = 4;
    let (minter_addr, _config) = setup_minter_contract(&mut router, &creator, num_tokens).unwrap();
    // set to genesis mint start time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME).unwrap();

    // try mint_for, test unauthorized
    let mint_for_msg = ExecuteMsg::MintFor {
        token_id: 1,
        recipient: buyer.clone(),
    };
    let err = router
        .execute_contract(
            buyer.clone(),
            minter_addr.clone(),
            &mint_for_msg,
            &coins(PRICE, NATIVE_DENOM),
        )
        .unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        ContractError::Unauthorized("Sender is not an admin".to_string()).to_string(),
    );

    // test token id already sold
    // 1. mint token_id 0
    // 2. mint_for token_id 0
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(PRICE, NATIVE_DENOM),
    );
    assert!(res.is_ok());

    // minter contract should have no balance
    let minter_balance = router
        .wrap()
        .query_all_balances(minter_addr.clone())
        .unwrap();
    assert_eq!(0, minter_balance.len());

    let token_id = 0;
    let mint_for_msg = ExecuteMsg::MintFor {
        token_id,
        recipient: buyer.clone(),
    };
    let err = router
        .execute_contract(
            creator.clone(),
            minter_addr.clone(),
            &mint_for_msg,
            &coins(PRICE, NATIVE_DENOM),
        )
        .unwrap_err();
    assert_eq!(
        ContractError::TokenIdAlreadySold { token_id }.to_string(),
        err.source().unwrap().to_string()
    );
    let mintable_num_tokens_response: MintableNumTokensResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &QueryMsg::MintableNumTokens {})
        .unwrap();
    assert_eq!(mintable_num_tokens_response.count, 3);

    // test mint_for token_id 2 then normal mint
    let token_id = 2;
    let mint_for_msg = ExecuteMsg::MintFor {
        token_id,
        recipient: buyer,
    };
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &mint_for_msg,
        &coins(PRICE, NATIVE_DENOM),
    );
    assert!(res.is_ok());

    let num_mints = 2;
    let batch_mint_msg = ExecuteMsg::BatchMint { num_mints };
    let res = router.execute_contract(
        creator,
        minter_addr.clone(),
        &batch_mint_msg,
        &coins(PRICE * num_mints as u128, NATIVE_DENOM),
    );
    assert!(res.is_ok());
    let mintable_num_tokens_response: MintableNumTokensResponse = router
        .wrap()
        .query_wasm_smart(minter_addr, &QueryMsg::MintableNumTokens {})
        .unwrap();
    assert_eq!(mintable_num_tokens_response.count, 0);
}

#[test]
fn test_start_time_before_genesis() {
    let mut router = custom_mock_app();
    let (creator, _) = setup_accounts(&mut router).unwrap();
    let num_tokens = 10;

    // Upload contract code
    let sg721_code_id = router.store_code(contract_sg721());
    let minter_code_id = router.store_code(contract_minter());
    let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);

    // Instantiate sale contract
    let msg = InstantiateMsg {
        unit_price: coin(PRICE, NATIVE_DENOM),
        num_tokens,
        start_time: Some(Expiration::AtTime(Timestamp::from_nanos(
            GENESIS_MINT_START_TIME - 100,
        ))),
        per_address_limit: None,
        batch_mint_limit: None,
        whitelist: None,
        base_token_uri: "ipfs://QmYxw1rURvnbQbBRTfmVaZtxSrkrfsbodNzibgBrVrUrtN".to_string(),
        sg721_code_id,
        sg721_instantiate_msg: Sg721InstantiateMsg {
            name: String::from("TEST"),
            symbol: String::from("TEST"),
            minter: creator.to_string(),
            config: Some(Config {
                contract_uri: Some(String::from("ipfs://url.json")),
                creator: Some(creator.clone()),
                royalties: Some(RoyaltyInfo {
                    payment_address: creator.clone(),
                    share: Decimal::percent(10),
                }),
            }),
        },
    };
    let minter_addr = router
        .instantiate_contract(minter_code_id, creator, &msg, &creation_fee, "Minter", None)
        .unwrap();

    let res: StartTimeResponse = router
        .wrap()
        .query_wasm_smart(minter_addr, &QueryMsg::StartTime {})
        .unwrap();
    assert_eq!(
        res.start_time,
        "expiration time: ".to_owned()
            + &Timestamp::from_nanos(GENESIS_MINT_START_TIME).to_string()
    );
}

#[test]
fn unhappy_path() {
    let mut router = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut router).unwrap();
    let num_tokens: u64 = 1;
    let (minter_addr, _config) = setup_minter_contract(&mut router, &creator, num_tokens).unwrap();

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
    let res = router.execute_contract(buyer, minter_addr, &mint_msg, &coins(PRICE, "uatom"));
    assert!(res.is_err());
}
