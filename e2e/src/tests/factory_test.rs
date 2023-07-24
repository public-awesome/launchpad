use assert_matches::assert_matches;
use cosm_orc::orchestrator::cosm_orc::tokio_block;
use cosm_orc::orchestrator::error::CosmwasmError::TxError;
use cosm_orc::orchestrator::error::ProcessError;
use cosm_orc::orchestrator::Coin as OrcCoin;
use cosm_orc::orchestrator::ExecReq;
use sg2::msg::Sg2ExecuteMsg;
use std::collections::HashMap;
use std::env;
use std::time::Duration;
use test_context::test_context;

use crate::helpers::{
    chain::Chain,
    helper::{
        create_minter_msg, gen_users, instantiate_factory, latest_block_time, CREATION_FEE,
        FACTORY_NAME, MAX_TOKENS, MINT_PRICE,
    },
};

#[test_context(Chain)]
#[test]
#[ignore]
fn test_instantiate_factory(chain: &mut Chain) {
    let user = chain.cfg.users[0].clone();
    instantiate_factory(chain, user.account.address, &user.key).unwrap();
}

#[test_context(Chain)]
#[test]
#[ignore]
fn test_create_minter(chain: &mut Chain) {
    if env::var("ENABLE_MAX_COLLECTION").is_err() {
        return;
    }

    let denom = chain.cfg.orc_cfg.chain_cfg.denom.clone();
    let user = chain.cfg.users[0].clone();
    let user_addr = &user.account.address;

    instantiate_factory(chain, user_addr.to_string(), &user.key).unwrap();

    let start_time = latest_block_time(chain).plus_seconds(60);

    let minter_msg = Sg2ExecuteMsg::CreateMinter(create_minter_msg(
        chain,
        user_addr.to_string(),
        MAX_TOKENS,
        50,
        start_time,
        None,
    ));

    // requires fee
    let err = chain
        .orc
        .execute(
            FACTORY_NAME,
            "factory_exec_minter_inst_no_fee_err",
            &minter_msg,
            &user.key,
            vec![],
        )
        .unwrap_err();
    assert_matches!(err, ProcessError::CosmwasmError(TxError(..)));
    assert!(err.to_string().contains("No funds sent"));

    // requires exact fee
    let err = chain
        .orc
        .execute(
            FACTORY_NAME,
            "factory_exec_minter_inst_exact_fee_err",
            &minter_msg,
            &user.key,
            vec![OrcCoin {
                amount: 50_000_000,
                denom: denom.parse().unwrap(),
            }],
        )
        .unwrap_err();
    assert_matches!(err, ProcessError::CosmwasmError(TxError(..)));
    assert!(err.to_string().contains("Insufficient fee"));

    let start_time = latest_block_time(chain).plus_seconds(5);

    let minter_msg = Sg2ExecuteMsg::CreateMinter(create_minter_msg(
        chain,
        user_addr.to_string(),
        MAX_TOKENS,
        50,
        start_time,
        None,
    ));

    let res = chain
        .orc
        .execute(
            FACTORY_NAME,
            "factory_exec_minter_inst",
            &minter_msg,
            &user.key,
            vec![OrcCoin {
                amount: CREATION_FEE,
                denom: denom.parse().unwrap(),
            }],
        )
        .unwrap();

    let tags = res
        .res
        .find_event_tags("instantiate".to_string(), "_contract_address".to_string());

    let (minter_addr, sg721_addr) = (tags[0].value.to_string(), tags[1].value.to_string());
    assert!(!minter_addr.trim().is_empty());
    assert!(!sg721_addr.trim().is_empty());

    // generate 200 user keys and send them all enough money to each mint 50 tokens + gas
    let users = gen_users(chain, 200, MINT_PRICE * 52, None);

    let init_balance = tokio_block(
        chain
            .orc
            .client
            .bank_query_balance(user_addr.parse().unwrap(), denom.parse().unwrap()),
    )
    .unwrap()
    .balance;

    // Sleep to ensure we can start minting
    chain
        .orc
        .poll_for_n_secs(6, Duration::from_millis(20_000))
        .unwrap();

    let mut total_mints = 0;
    let mut mints: HashMap<String, bool> = HashMap::new();

    let num_users = users.len() as u32;

    // Batch mint all tokens:
    chain
        .orc
        .contract_map
        .add_address("minter", minter_addr)
        .unwrap();

    for user in &users {
        let mut reqs = vec![];
        for _ in 0..MAX_TOKENS / num_users {
            total_mints += 1;
            reqs.push(ExecReq {
                contract_name: "minter".to_string(),
                msg: Box::new(vending_minter::msg::ExecuteMsg::Mint {}),
                funds: vec![OrcCoin {
                    amount: MINT_PRICE,
                    denom: denom.parse().unwrap(),
                }],
            });
        }

        let res = chain
            .orc
            .execute_batch("minter_batch_exec_mint_token", reqs, user)
            .unwrap();

        let token_ids = res
            .res
            .find_event_tags("wasm".to_string(), "token_id".to_string());

        let mut map = HashMap::new();
        for id in token_ids {
            map.insert(&id.value, true);
        }

        for (token_id, _) in map {
            assert_eq!(mints.get(token_id), None);
            mints.insert(token_id.to_string(), true);
        }
    }

    for n in 1..=MAX_TOKENS {
        assert_eq!(mints.get(&n.to_string()), Some(&true));
    }
    assert_eq!(total_mints, MAX_TOKENS);

    let balance = tokio_block(
        chain
            .orc
            .client
            .bank_query_balance(user_addr.parse().unwrap(), denom.parse().unwrap()),
    )
    .unwrap()
    .balance;

    // 10k x MINT_PRICE = 1M STARS x 0.9 (10% fee)
    assert_eq!(balance.amount, init_balance.amount + 900_000_000_000);

    // Cannot mint more:
    let res = chain.orc.execute(
        "minter",
        "minter_exec_mint_token_err",
        &vending_minter::msg::ExecuteMsg::Mint {},
        &chain.cfg.users[1].key,
        vec![OrcCoin {
            amount: MINT_PRICE,
            denom: denom.parse().unwrap(),
        }],
    );

    let err = res.unwrap_err();
    assert_matches!(err, ProcessError::CosmwasmError(TxError(..)));
    assert!(err.to_string().contains("Sold out"));
}

#[test_context(Chain)]
#[test]
#[ignore]
fn test_start_trading_time(chain: &mut Chain) {
    let denom = chain.cfg.orc_cfg.chain_cfg.denom.clone();
    let user = chain.cfg.users[0].clone();
    let user_addr = &user.account.address;

    let initial_total_supply =
        tokio_block(chain.orc.client.bank_query_supply(denom.parse().unwrap()))
            .unwrap()
            .balance;

    instantiate_factory(chain, user_addr.to_string(), &user.key).unwrap();

    let start_time = latest_block_time(chain).plus_seconds(5);

    let minter_msg = Sg2ExecuteMsg::CreateMinter(create_minter_msg(
        chain,
        user_addr.to_string(),
        1000,
        10,
        start_time,
        Some(start_time.plus_seconds(60 * 60)),
    ));

    let res = chain
        .orc
        .execute(
            FACTORY_NAME,
            "factory_exec_minter_inst_w_trading_time",
            &minter_msg,
            &user.key,
            vec![OrcCoin {
                amount: CREATION_FEE,
                denom: denom.parse().unwrap(),
            }],
        )
        .unwrap();

    let tags = res
        .res
        .find_event_tags("instantiate".to_string(), "_contract_address".to_string());

    let (minter_addr, sg721_addr) = (tags[0].value.clone(), tags[1].value.clone());
    assert!(!minter_addr.trim().is_empty());
    assert!(!sg721_addr.trim().is_empty());

    let mut total_mints = 0;
    let mut total_fairburn_fees = 0;

    let fair_burn_fees = res
        .res
        .find_event_tags("fund_fairburn_pool".to_string(), "amount".to_string());

    let amount = fair_burn_fees[0].value.split(&denom).collect::<Vec<&str>>()[0];
    total_fairburn_fees += amount.parse::<u128>().unwrap();

    let users = gen_users(chain, 20, MINT_PRICE * 12, None);
    let num_users = users.len() as u32;

    chain
        .orc
        .contract_map
        .add_address("minter", minter_addr)
        .unwrap();

    // Sleep to ensure we can start minting
    chain
        .orc
        .poll_for_n_secs(6, Duration::from_millis(20_000))
        .unwrap();

    let init_balance = tokio_block(
        chain
            .orc
            .client
            .bank_query_balance(user_addr.parse().unwrap(), denom.parse().unwrap()),
    )
    .unwrap()
    .balance;

    for user in &users {
        let mut reqs = vec![];
        for _ in 0..100 / num_users {
            total_mints += 1;
            reqs.push(ExecReq {
                contract_name: "minter".to_string(),
                msg: Box::new(vending_minter::msg::ExecuteMsg::Mint {}),
                funds: vec![OrcCoin {
                    amount: MINT_PRICE,
                    denom: denom.parse().unwrap(),
                }],
            });
        }

        let res = chain
            .orc
            .execute_batch("minter_batch_exec_mint_token_w_trading_time", reqs, user)
            .unwrap();

        let fair_burn_fees = res
            .res
            .find_event_tags("fund_fairburn_pool".to_string(), "amount".to_string());

        for fee in fair_burn_fees {
            let amount = fee.value.split(&denom).collect::<Vec<&str>>()[0];
            total_fairburn_fees += amount.parse::<u128>().unwrap();
        }
    }

    assert_eq!(total_mints, 100);

    let balance = tokio_block(
        chain
            .orc
            .client
            .bank_query_balance(user_addr.parse().unwrap(), denom.parse().unwrap()),
    )
    .unwrap()
    .balance;

    // 100 x MINT_PRICE = 10k STARS x 0.9 (10% fee)
    assert_eq!(balance.amount, init_balance.amount + 9_000_000_000);

    // fairburn fees
    // half of the 10% fees should be sent to fairburn pool
    // 500STARS + 500STARS initially sent for collection creation fee
    assert_eq!(total_fairburn_fees, 1_000_000_000);

    let total_supply = tokio_block(chain.orc.client.bank_query_supply(denom.parse().unwrap()))
        .unwrap()
        .balance;

    // the other half burned
    assert_eq!(
        initial_total_supply.amount - 1_000_000_000,
        total_supply.amount
    );
}

#[test_context(Chain)]
#[test]
#[ignore]
fn test_invalid_start_trading_time(chain: &mut Chain) {
    let denom = chain.cfg.orc_cfg.chain_cfg.denom.clone();
    let user = chain.cfg.users[0].clone();
    let user_addr = &user.account.address;

    instantiate_factory(chain, user_addr.to_string(), &user.key).unwrap();

    let start_time = latest_block_time(chain).plus_seconds(100_000);

    let minter_msg = Sg2ExecuteMsg::CreateMinter(create_minter_msg(
        chain,
        user_addr.to_string(),
        1000,
        10,
        start_time,
        Some(start_time.plus_seconds(60 * 60 * 24 * 365)),
    ));

    let res = chain.orc.execute(
        FACTORY_NAME,
        "factory_exec_minter_inst_w_trading_time_err",
        &minter_msg,
        &user.key,
        vec![OrcCoin {
            amount: CREATION_FEE,
            denom: denom.parse().unwrap(),
        }],
    );

    let err = res.unwrap_err();
    assert_matches!(err, ProcessError::CosmwasmError(TxError(..)));
    assert!(err.to_string().contains("InvalidStartTradingTime"));
}
