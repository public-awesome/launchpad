use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
use cosmwasm_std::{coin, coins, Addr, Decimal, Empty, Timestamp, Uint128};
use cosmwasm_std::{Api, Coin};
use cw721::{Cw721QueryMsg, OwnerOfResponse};
use cw721_base::ExecuteMsg as Cw721ExecuteMsg;
use cw_multi_test::{BankSudo, Contract, ContractWrapper, Executor, SudoMsg};
use sg721::msg::{InstantiateMsg as Sg721InstantiateMsg, RoyaltyInfoResponse};
use sg721::state::CollectionInfo;
use sg_multi_test::StargazeApp;
use sg_std::{StargazeMsgWrapper, GENESIS_MINT_START_TIME, NATIVE_DENOM};
use whitelist::msg::InstantiateMsg as WhitelistInstantiateMsg;
use whitelist::msg::{AddMembersMsg, ExecuteMsg as WhitelistExecuteMsg};

use crate::contract::instantiate;
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, MintCountResponse, MintPriceResponse,
    MintableNumTokensResponse, QueryMsg, StartTimeResponse,
};
use crate::ContractError;

const CREATION_FEE: u128 = 1_000_000_000;
const INITIAL_BALANCE: u128 = 2_000_000_000;

const UNIT_PRICE: u128 = 100_000_000;
const MINT_FEE: u128 = 10_000_000;
const MAX_TOKEN_LIMIT: u32 = 10000;
const WHITELIST_AMOUNT: u128 = 66_000_000;
const WL_PER_ADDRESS_LIMIT: u32 = 1;
const ADMIN_MINT_PRICE: u128 = 15_000_000;

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

fn setup_whitelist_contract(router: &mut StargazeApp, creator: &Addr) -> Addr {
    let whitelist_code_id = router.store_code(contract_whitelist());

    let msg = WhitelistInstantiateMsg {
        members: vec![],
        start_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME + 100),
        end_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10000000),
        unit_price: coin(WHITELIST_AMOUNT, NATIVE_DENOM),
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

// Upload contract code and instantiate minter contract
fn setup_minter_contract(
    router: &mut StargazeApp,
    creator: &Addr,
    num_tokens: u32,
) -> (Addr, ConfigResponse) {
    // Upload contract code
    let sg721_code_id = router.store_code(contract_sg721());
    let minter_code_id = router.store_code(contract_minter());
    let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);

    // Instantiate minter contract
    let msg = InstantiateMsg {
        unit_price: coin(UNIT_PRICE, NATIVE_DENOM),
        num_tokens,
        start_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME),
        per_address_limit: 5,
        whitelist: None,
        base_token_uri: "ipfs://QmYxw1rURvnbQbBRTfmVaZtxSrkrfsbodNzibgBrVrUrtN".to_string(),
        sg721_code_id,
        sg721_instantiate_msg: Sg721InstantiateMsg {
            name: String::from("TEST"),
            symbol: String::from("TEST"),
            minter: creator.to_string(),
            collection_info: CollectionInfo {
                creator: creator.to_string(),
                description: String::from("Stargaze Monkeys"),
                image: "https://example.com/image.png".to_string(),
                external_link: Some("https://example.com/external.html".to_string()),
                royalty_info: Some(RoyaltyInfoResponse {
                    payment_address: creator.to_string(),
                    share: Decimal::percent(10),
                }),
            },
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

    (minter_addr, config)
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
fn setup_block_time(router: &mut StargazeApp, nanos: u64) {
    let mut block = router.block_info();
    block.time = Timestamp::from_nanos(nanos);
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
    let res = deps.api.addr_validate(&(*addr));
    assert!(res.is_ok());

    // 0 per address limit returns error
    let info = mock_info("creator", &coins(INITIAL_BALANCE, NATIVE_DENOM));
    let msg = InstantiateMsg {
        unit_price: coin(UNIT_PRICE, NATIVE_DENOM),
        num_tokens: 100,
        start_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME),
        per_address_limit: 0,
        whitelist: None,
        base_token_uri: "ipfs://QmYxw1rURvnbQbBRTfmVaZtxSrkrfsbodNzibgBrVrUrtN".to_string(),
        sg721_code_id: 1,
        sg721_instantiate_msg: Sg721InstantiateMsg {
            name: String::from("TEST"),
            symbol: String::from("TEST"),
            minter: info.sender.to_string(),
            collection_info: CollectionInfo {
                creator: info.sender.to_string(),
                description: String::from("Stargaze Monkeys"),
                image: "https://example.com/image.png".to_string(),
                external_link: Some("https://example.com/external.html".to_string()),
                royalty_info: Some(RoyaltyInfoResponse {
                    payment_address: info.sender.to_string(),
                    share: Decimal::percent(10),
                }),
            },
        },
    };
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    // Invalid uri returns error
    let info = mock_info("creator", &coins(INITIAL_BALANCE, NATIVE_DENOM));
    let msg = InstantiateMsg {
        unit_price: coin(UNIT_PRICE, NATIVE_DENOM),
        num_tokens: 100,
        start_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME),
        per_address_limit: 5,
        whitelist: None,
        base_token_uri: "https://QmYxw1rURvnbQbBRTfmVaZtxSrkrfsbodNzibgBrVrUrtN".to_string(),
        sg721_code_id: 1,
        sg721_instantiate_msg: Sg721InstantiateMsg {
            name: String::from("TEST"),
            symbol: String::from("TEST"),
            minter: info.sender.to_string(),
            collection_info: CollectionInfo {
                creator: info.sender.to_string(),
                description: String::from("Stargaze Monkeys"),
                image: "https://example.com/image.png".to_string(),
                external_link: Some("https://example.com/external.html".to_string()),
                royalty_info: Some(RoyaltyInfoResponse {
                    payment_address: info.sender.to_string(),
                    share: Decimal::percent(10),
                }),
            },
        },
    };
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    // Invalid denom returns error
    let wrong_denom = "uosmo";
    let info = mock_info("creator", &coins(INITIAL_BALANCE, NATIVE_DENOM));
    let msg = InstantiateMsg {
        unit_price: coin(UNIT_PRICE, wrong_denom),
        num_tokens: 100,
        start_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME),
        per_address_limit: 5,
        whitelist: None,
        base_token_uri: "ipfs://QmYxw1rURvnbQbBRTfmVaZtxSrkrfsbodNzibgBrVrUrtN".to_string(),
        sg721_code_id: 1,
        sg721_instantiate_msg: Sg721InstantiateMsg {
            name: String::from("TEST"),
            symbol: String::from("TEST"),
            minter: info.sender.to_string(),
            collection_info: CollectionInfo {
                creator: info.sender.to_string(),
                description: String::from("Stargaze Monkeys"),
                image: "https://example.com/image.png".to_string(),
                external_link: Some("https://example.com/external.html".to_string()),
                royalty_info: Some(RoyaltyInfoResponse {
                    payment_address: info.sender.to_string(),
                    share: Decimal::percent(10),
                }),
            },
        },
    };
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    // Insufficient mint price returns error
    let info = mock_info("creator", &coins(INITIAL_BALANCE, NATIVE_DENOM));
    let msg = InstantiateMsg {
        unit_price: coin(1, NATIVE_DENOM),
        num_tokens: 100,
        start_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME),
        per_address_limit: 5,
        whitelist: None,
        base_token_uri: "ipfs://QmYxw1rURvnbQbBRTfmVaZtxSrkrfsbodNzibgBrVrUrtN".to_string(),
        sg721_code_id: 1,
        sg721_instantiate_msg: Sg721InstantiateMsg {
            name: String::from("TEST"),
            symbol: String::from("TEST"),
            minter: info.sender.to_string(),
            collection_info: CollectionInfo {
                creator: info.sender.to_string(),
                description: String::from("Stargaze Monkeys"),
                image: "https://example.com/image.png".to_string(),
                external_link: Some("https://example.com/external.html".to_string()),
                royalty_info: Some(RoyaltyInfoResponse {
                    payment_address: info.sender.to_string(),
                    share: Decimal::percent(10),
                }),
            },
        },
    };
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    // Over max token limit
    let info = mock_info("creator", &coins(INITIAL_BALANCE, NATIVE_DENOM));
    let msg = InstantiateMsg {
        unit_price: coin(UNIT_PRICE, NATIVE_DENOM),
        num_tokens: (MAX_TOKEN_LIMIT + 1),
        start_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME),
        per_address_limit: 5,
        whitelist: None,
        base_token_uri: "ipfs://QmYxw1rURvnbQbBRTfmVaZtxSrkrfsbodNzibgBrVrUrtN".to_string(),
        sg721_code_id: 1,
        sg721_instantiate_msg: Sg721InstantiateMsg {
            name: String::from("TEST"),
            symbol: String::from("TEST"),
            minter: info.sender.to_string(),
            collection_info: CollectionInfo {
                creator: info.sender.to_string(),
                description: String::from("Stargaze Monkeys"),
                image: "https://example.com/image.png".to_string(),
                external_link: Some("https://example.com/external.html".to_string()),
                royalty_info: Some(RoyaltyInfoResponse {
                    payment_address: info.sender.to_string(),
                    share: Decimal::percent(10),
                }),
            },
        },
    };
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    // Under min token limit
    let info = mock_info("creator", &coins(INITIAL_BALANCE, NATIVE_DENOM));
    let msg = InstantiateMsg {
        unit_price: coin(UNIT_PRICE, NATIVE_DENOM),
        num_tokens: 0,
        start_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME),
        per_address_limit: 5,
        whitelist: None,
        base_token_uri: "ipfs://QmYxw1rURvnbQbBRTfmVaZtxSrkrfsbodNzibgBrVrUrtN".to_string(),
        sg721_code_id: 1,
        sg721_instantiate_msg: Sg721InstantiateMsg {
            name: String::from("TEST"),
            symbol: String::from("TEST"),
            minter: info.sender.to_string(),
            collection_info: CollectionInfo {
                creator: info.sender.to_string(),
                description: String::from("Stargaze Monkeys"),
                image: "https://example.com/image.png".to_string(),
                external_link: Some("https://example.com/external.html".to_string()),
                royalty_info: Some(RoyaltyInfoResponse {
                    payment_address: info.sender.to_string(),
                    share: Decimal::percent(10),
                }),
            },
        },
    };
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();
}

#[test]
fn happy_path() {
    let mut router = custom_mock_app();
    setup_block_time(&mut router, GENESIS_MINT_START_TIME - 1);
    let (creator, buyer) = setup_accounts(&mut router);
    let num_tokens = 2;
    let (minter_addr, config) = setup_minter_contract(&mut router, &creator, num_tokens);

    // Default start time genesis mint time
    let res: StartTimeResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &QueryMsg::StartTime {})
        .unwrap();
    assert_eq!(
        res.start_time,
        Timestamp::from_nanos(GENESIS_MINT_START_TIME).to_string()
    );

    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 1);

    // Fail with incorrect tokens
    let mint_msg = ExecuteMsg::Mint {};
    let err = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(UNIT_PRICE + 100, NATIVE_DENOM),
    );
    assert!(err.is_err());

    // Succeeds if funds are sent
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(UNIT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_ok());

    // Balances are correct
    // The creator should get the unit price - mint fee for the mint above
    let creator_balances = router.wrap().query_all_balances(creator.clone()).unwrap();
    assert_eq!(creator_balances, coins(INITIAL_BALANCE, NATIVE_DENOM));
    // The buyer's tokens should reduce by unit price
    let buyer_balances = router.wrap().query_all_balances(buyer.clone()).unwrap();
    assert_eq!(
        buyer_balances,
        coins(INITIAL_BALANCE - UNIT_PRICE, NATIVE_DENOM)
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

    // Minter contract should have a balance
    let minter_balance = router
        .wrap()
        .query_all_balances(minter_addr.clone())
        .unwrap();
    assert_eq!(1, minter_balance.len());
    assert_eq!(minter_balance[0].amount.u128(), UNIT_PRICE - MINT_FEE);

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
        &coins_for_msg(Coin {
            amount: Uint128::from(UNIT_PRICE),
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
#[test]
fn mint_count_query() {
    let mut router = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut router);
    let num_tokens = 10;
    let (minter_addr, config) = setup_minter_contract(&mut router, &creator, num_tokens);
    let sg721_addr = Addr::unchecked(config.sg721_address);
    let whitelist_addr = setup_whitelist_contract(&mut router, &creator);
    const EXPIRATION_TIME: Timestamp = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10_000);

    // Set block to before genesis mint start time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME - 1000);

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

    setup_block_time(&mut router, GENESIS_MINT_START_TIME);

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
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 20_000);

    // Public mint succeeds
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(UNIT_PRICE, NATIVE_DENOM),
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

    // Buyer transfers NFT to creator
    let transfer_msg: Cw721ExecuteMsg<Empty> = Cw721ExecuteMsg::TransferNft {
        recipient: creator.to_string(),
        token_id: "1".to_string(),
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
        &coins(UNIT_PRICE, NATIVE_DENOM),
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
    let (minter_addr, _) = setup_minter_contract(&mut router, &creator, num_tokens);
    let whitelist_addr = setup_whitelist_contract(&mut router, &creator);

    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 101);

    // set whitelist in minter contract
    let set_whitelist_msg = ExecuteMsg::SetWhitelist {
        whitelist: whitelist_addr.to_string(),
    };
    router
        .execute_contract(
            creator.clone(),
            minter_addr,
            &set_whitelist_msg,
            &coins(UNIT_PRICE, NATIVE_DENOM),
        )
        .unwrap_err();
}

#[test]
fn whitelist_can_update_before_start() {
    let mut router = custom_mock_app();
    let (creator, _) = setup_accounts(&mut router);
    let num_tokens = 1;
    let (minter_addr, _) = setup_minter_contract(&mut router, &creator, num_tokens);
    let whitelist_addr = setup_whitelist_contract(&mut router, &creator);

    setup_block_time(&mut router, GENESIS_MINT_START_TIME - 1000);

    // set whitelist in minter contract
    let set_whitelist_msg = ExecuteMsg::SetWhitelist {
        whitelist: whitelist_addr.to_string(),
    };
    router
        .execute_contract(
            creator.clone(),
            minter_addr.clone(),
            &set_whitelist_msg,
            &coins(UNIT_PRICE, NATIVE_DENOM),
        )
        .unwrap();

    // can set twice before starting
    router
        .execute_contract(
            creator.clone(),
            minter_addr,
            &set_whitelist_msg,
            &coins(UNIT_PRICE, NATIVE_DENOM),
        )
        .unwrap();
}

#[test]
fn whitelist_access_len_add_remove_expiration() {
    let mut router = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut router);
    let num_tokens = 1;
    let (minter_addr, config) = setup_minter_contract(&mut router, &creator, num_tokens);
    let sg721_addr = config.sg721_address;
    let whitelist_addr = setup_whitelist_contract(&mut router, &creator);
    const AFTER_GENESIS_TIME: Timestamp = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 100);

    // Set to just before genesis mint start time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME - 10);

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
        &coins(UNIT_PRICE, NATIVE_DENOM),
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
            &coins(UNIT_PRICE, NATIVE_DENOM),
        )
        .unwrap_err();

    setup_block_time(&mut router, GENESIS_MINT_START_TIME);

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
        coin(UNIT_PRICE, NATIVE_DENOM),
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
    let transfer_msg: Cw721ExecuteMsg<Empty> = Cw721ExecuteMsg::TransferNft {
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
        &coins(UNIT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_err());
}

#[test]
fn before_start_time() {
    let mut router = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut router);
    let num_tokens = 1;
    let (minter_addr, _) = setup_minter_contract(&mut router, &creator, num_tokens);

    // Set to before genesis mint start time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME - 10);

    // Set start_time fails if not admin
    let start_time_msg = ExecuteMsg::UpdateStartTime(Timestamp::from_nanos(0));
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &start_time_msg,
        &coins(UNIT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_err());

    // Buyer can't mint before start_time
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(UNIT_PRICE, NATIVE_DENOM),
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
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 10_000_000);

    // Mint succeeds
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer,
        minter_addr,
        &mint_msg,
        &coins(UNIT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_ok());
}

#[test]
fn check_per_address_limit() {
    let mut router = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut router);
    let num_tokens = 2;
    let (minter_addr, _config) = setup_minter_contract(&mut router, &creator, num_tokens);

    // Set to genesis mint start time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME);

    // Set limit, check unauthorized
    let per_address_limit_msg = ExecuteMsg::UpdatePerAddressLimit {
        per_address_limit: 30,
    };
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &per_address_limit_msg,
        &coins(UNIT_PRICE, NATIVE_DENOM),
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
        &coins(UNIT_PRICE, NATIVE_DENOM),
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
        &coins(UNIT_PRICE, NATIVE_DENOM),
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
        &coins(UNIT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_ok());

    // First mint succeeds
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(UNIT_PRICE, NATIVE_DENOM),
    );

    assert!(res.is_ok());

    // Second mint fails from exceeding per address limit
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer,
        minter_addr,
        &mint_msg,
        &coins(UNIT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_err());
}

#[test]
fn mint_for_token_id_addr() {
    let mut router = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut router);
    let num_tokens = 4;
    let (minter_addr, _config) = setup_minter_contract(&mut router, &creator, num_tokens);

    // Set to genesis mint start time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME);

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
    // 1. mint token_id 1
    // 2. mint_for token_id 1
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(UNIT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_ok());

    // Minter contract should have a balance
    let minter_balance = router
        .wrap()
        .query_all_balances(minter_addr.clone())
        .unwrap();
    assert_eq!(minter_balance[0].amount.u128(), UNIT_PRICE - MINT_FEE);
    assert_eq!(1, minter_balance.len());

    // Mint fails, invalid token_id
    let token_id = 0;
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
    let token_id = 1;
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
        ContractError::TokenIdAlreadySold { token_id }.to_string(),
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
                amount: Uint128::from(ADMIN_MINT_PRICE - 1),
                denom: NATIVE_DENOM.to_string(),
            }),
        )
        .unwrap_err();
    assert_eq!(
        ContractError::IncorrectPaymentAmount(
            coin(ADMIN_MINT_PRICE - 1, NATIVE_DENOM.to_string()),
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

    let mintable_num_tokens_response: MintableNumTokensResponse = router
        .wrap()
        .query_wasm_smart(minter_addr, &QueryMsg::MintableNumTokens {})
        .unwrap();
    assert_eq!(mintable_num_tokens_response.count, 2);
}

#[test]
fn test_start_time_before_genesis() {
    let mut router = custom_mock_app();
    let (creator, _) = setup_accounts(&mut router);
    let num_tokens = 10;

    // Upload contract code
    let sg721_code_id = router.store_code(contract_sg721());
    let minter_code_id = router.store_code(contract_minter());
    let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);

    // Instantiate sale contract
    let msg = InstantiateMsg {
        unit_price: coin(UNIT_PRICE, NATIVE_DENOM),
        num_tokens,
        start_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME),
        per_address_limit: 5,
        whitelist: None,
        base_token_uri: "ipfs://QmYxw1rURvnbQbBRTfmVaZtxSrkrfsbodNzibgBrVrUrtN".to_string(),
        sg721_code_id,
        sg721_instantiate_msg: Sg721InstantiateMsg {
            name: String::from("TEST"),
            symbol: String::from("TEST"),
            minter: creator.to_string(),
            collection_info: CollectionInfo {
                creator: creator.to_string(),
                description: String::from("Stargaze Monkeys"),
                image: "https://example.com/image.png".to_string(),
                external_link: Some("https://example.com/external.html".to_string()),
                royalty_info: Some(RoyaltyInfoResponse {
                    payment_address: creator.to_string(),
                    share: Decimal::percent(10),
                }),
            },
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
        Timestamp::from_nanos(GENESIS_MINT_START_TIME).to_string()
    );
}

#[test]
fn test_update_start_time() {
    let mut router = custom_mock_app();
    let (creator, _) = setup_accounts(&mut router);
    let num_tokens = 10;

    // Upload contract code
    let sg721_code_id = router.store_code(contract_sg721());
    let minter_code_id = router.store_code(contract_minter());
    let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);

    // Instantiate sale contract
    let msg = InstantiateMsg {
        unit_price: coin(UNIT_PRICE, NATIVE_DENOM),
        num_tokens,
        start_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME),
        per_address_limit: 5,
        whitelist: None,
        base_token_uri: "ipfs://QmYxw1rURvnbQbBRTfmVaZtxSrkrfsbodNzibgBrVrUrtN".to_string(),
        sg721_code_id,
        sg721_instantiate_msg: Sg721InstantiateMsg {
            name: String::from("TEST"),
            symbol: String::from("TEST"),
            minter: creator.to_string(),
            collection_info: CollectionInfo {
                creator: creator.to_string(),
                description: String::from("Stargaze Monkeys"),
                image: "https://example.com/image.png".to_string(),
                external_link: Some("https://example.com/external.html".to_string()),
                royalty_info: None,
            },
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

    // Public mint has started
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 100);

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
    let num_tokens = 10;

    // Upload contract code
    let sg721_code_id = router.store_code(contract_sg721());
    let minter_code_id = router.store_code(contract_minter());
    let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);

    // Instantiate sale contract before genesis mint
    let mut msg = InstantiateMsg {
        unit_price: coin(UNIT_PRICE, NATIVE_DENOM),
        num_tokens,
        start_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME - 100),
        per_address_limit: 5,
        whitelist: None,
        base_token_uri: "ipfs://QmYxw1rURvnbQbBRTfmVaZtxSrkrfsbodNzibgBrVrUrtN".to_string(),
        sg721_code_id,
        sg721_instantiate_msg: Sg721InstantiateMsg {
            name: String::from("TEST"),
            symbol: String::from("TEST"),
            minter: creator.to_string(),
            collection_info: CollectionInfo {
                creator: creator.to_string(),
                description: String::from("Stargaze Monkeys"),
                image: "https://example.com/image.png".to_string(),
                external_link: Some("https://example.com/external.html".to_string()),
                royalty_info: None,
            },
        },
    };
    // set time before the start_time above
    setup_block_time(&mut router, GENESIS_MINT_START_TIME - 1000);

    let err = router
        .instantiate_contract(
            minter_code_id,
            creator.clone(),
            &msg,
            &creation_fee,
            "Minter",
            None,
        )
        .unwrap_err();
    assert_eq!(err.source().unwrap().to_string(), "BeforeGenesisTime");

    // move date after genesis mint
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 1000);

    // move start time after genesis but before current time
    msg.start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 500);

    router
        .instantiate_contract(
            minter_code_id,
            creator.clone(),
            &msg,
            &creation_fee,
            "Minter",
            None,
        )
        .unwrap_err();

    // position block time before the start time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 400);

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
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 500);

    // Update to a time after genesis and after current blocktime (GENESIS+400)
    let msg = ExecuteMsg::UpdateStartTime(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 450));
    let err = router
        .execute_contract(creator, minter_addr, &msg, &[])
        .unwrap_err();
    assert_eq!(err.source().unwrap().to_string(), "AlreadyStarted");
}

#[test]
fn unhappy_path() {
    let mut router = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut router);
    let num_tokens = 1;
    let (minter_addr, _config) = setup_minter_contract(&mut router, &creator, num_tokens);

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
    let res = router.execute_contract(buyer, minter_addr, &mint_msg, &coins(UNIT_PRICE, "uatom"));
    assert!(res.is_err());
}

#[test]
fn can_withdraw() {
    // create minter
    let mut router = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut router);
    let num_tokens = 4;
    let (minter_addr, _config) = setup_minter_contract(&mut router, &creator, num_tokens);

    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 1);

    // someone who isn't the creator cannot withdraw
    let withdraw_msg = ExecuteMsg::Withdraw {};
    router
        .execute_contract(buyer.clone(), minter_addr.clone(), &withdraw_msg, &[])
        .unwrap_err();

    // withdraw with a zero balance
    router
        .execute_contract(creator.clone(), minter_addr.clone(), &withdraw_msg, &[])
        .unwrap_err();

    // do a mint
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer,
        minter_addr.clone(),
        &mint_msg,
        &coins(UNIT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_ok());

    // Minter contract should have a balance
    let minter_balance = router
        .wrap()
        .query_all_balances(minter_addr.clone())
        .unwrap();
    assert_eq!(minter_balance[0].amount.u128(), UNIT_PRICE - MINT_FEE);

    // withdraw
    let res = router.execute_contract(creator.clone(), minter_addr.clone(), &withdraw_msg, &[]);
    assert!(res.is_ok());

    // Minter contract should have no balance
    let minter_balance = router.wrap().query_all_balances(minter_addr).unwrap();
    assert_eq!(0, minter_balance.len());

    // creator should have received their payment
    let creator_balances = router.wrap().query_all_balances(creator).unwrap();
    assert_eq!(
        creator_balances,
        coins(INITIAL_BALANCE + UNIT_PRICE - MINT_FEE, NATIVE_DENOM)
    );
}
