use assert_matches::assert_matches;
use cosm_orc::orchestrator::cosm_orc::tokio_block;
use cosm_orc::orchestrator::error::CosmwasmError::TxError;
use cosm_orc::orchestrator::error::ProcessError;
use cosm_orc::orchestrator::Coin as OrcCoin;
use cosm_orc::orchestrator::ExecReq;
use open_edition_factory::types::{NftData, NftMetadataType};
use open_edition_minter::msg::ExecuteMsg::{UpdatePerAddressLimit, UpdateStartTradingTime};
use sg2::msg::Sg2ExecuteMsg;
use std::collections::HashMap;
use std::time::Duration;
use test_context::test_context;

use crate::helpers::{
    chain::Chain,
    helper::{gen_users, latest_block_time},
    open_edition_minter_helpers::{
        create_minter_msg, instantiate_factory, CREATION_FEE, FACTORY_NAME, MAX_TOKENS, MINT_PRICE,
    },
};

#[test_context(Chain)]
#[test]
#[ignore]
fn test_open_edition_exec_functions(chain: &mut Chain) {
    let denom = chain.cfg.orc_cfg.chain_cfg.denom.clone();
    let user = chain.cfg.users[0].clone();
    let user_addr = &user.account.address;
    let dev = chain.cfg.users[1].clone();

    instantiate_factory(
        chain,
        user_addr.clone(),
        dev.account.address.clone(),
        &user.key,
    )
    .unwrap();

    let start_time = latest_block_time(chain).plus_seconds(10);
    let end_time = Some(latest_block_time(chain).plus_seconds(60));

    let valid_minter_msg = Sg2ExecuteMsg::CreateMinter(create_minter_msg(
        chain,
        None,
        user_addr.to_string(),
        MAX_TOKENS,
        start_time,
        end_time,
        None,
        None,
        NftData {
            nft_data_type: NftMetadataType::OffChainMetadata,
            extension: None,
            token_uri: Some("ipfs://...".to_string()),
        },
    ));

    let res = chain
        .orc
        .execute(
            FACTORY_NAME,
            "factory_exec_minter_inst",
            &valid_minter_msg,
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

    let (minter_addr, _sg721_addr) = (tags[0].value.to_string(), tags[1].value.to_string());

    // generate 10 user keys and send them all enough money to each mint 10 tokens (max)
    let users = gen_users(chain, 10, MINT_PRICE * MAX_TOKENS as u128 * 2u128, None);

    // Sleep to ensure we can start minting
    chain
        .orc
        .poll_for_n_secs(10, Duration::from_millis(20_000))
        .unwrap();

    let mut total_mints = 0;
    let mut mints: HashMap<String, bool> = HashMap::new();

    // Store minter
    chain
        .orc
        .contract_map
        .add_address("minter", minter_addr)
        .unwrap();

    // Change start trading time
    let res = chain
        .orc
        .execute(
            "minter",
            "minter_exec_update_trading_time",
            &UpdateStartTradingTime(Some(start_time.plus_seconds(10))),
            &user.key,
            vec![],
        )
        .unwrap();
    let tags_update = res
        .res
        .find_event_tags("wasm".to_string(), "action".to_string());
    assert_eq!(
        tags_update[0].value.to_string().trim(),
        "update_start_trading_time".to_string()
    );

    // Initial after changing time
    let init_balance = tokio_block(
        chain
            .orc
            .client
            .bank_query_balance(user_addr.parse().unwrap(), denom.parse().unwrap()),
    )
    .unwrap()
    .balance;

    // Batch mint 5 tokens / user:
    for user in &users {
        let mut reqs = vec![];
        for _ in 0..5 {
            total_mints += 1;
            reqs.push(ExecReq {
                contract_name: "minter".to_string(),
                msg: Box::new(open_edition_minter::msg::ExecuteMsg::Mint {}),
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

    for n in 1..=5 {
        assert_eq!(mints.get(&n.to_string()), Some(&true));
    }
    assert_eq!(total_mints, 5 * users.len() as u32);

    let balance = tokio_block(
        chain
            .orc
            .client
            .bank_query_balance(user_addr.parse().unwrap(), denom.parse().unwrap()),
    )
    .unwrap()
    .balance;

    // 5 * 10 * MINT_PRICE = 5k STARS x 0.9 (10% fee)
    assert_eq!(balance.amount, init_balance.amount + 4_500_000_000);

    // Mint To
    let _res = chain
        .orc
        .execute(
            "minter",
            "minter_exec_mint_to_token",
            &vending_minter::msg::ExecuteMsg::MintTo {
                recipient: dev.account.address.to_string(),
            },
            &user.key,
            vec![],
        )
        .unwrap();

    // Mint To -> not admin
    let res = chain.orc.execute(
        "minter",
        "minter_exec_mint_to_token_err",
        &vending_minter::msg::ExecuteMsg::MintTo {
            recipient: user.account.address.to_string(),
        },
        &dev.key,
        vec![],
    );
    assert!(res.is_err());

    // Purge -> not ended
    let res = chain.orc.execute(
        "minter",
        "minter_exec_purge_err",
        &vending_minter::msg::ExecuteMsg::Purge {},
        &dev.key,
        vec![],
    );
    assert!(res.is_err());

    // Change Per Address Limit to 5 -> Should now be able to mint anymore
    let res = chain
        .orc
        .execute(
            "minter",
            "minter_exec_update_per_addr_limit",
            &UpdatePerAddressLimit {
                per_address_limit: 5,
            },
            &user.key,
            vec![],
        )
        .unwrap();
    let tags_update = res
        .res
        .find_event_tags("wasm".to_string(), "action".to_string());
    assert_eq!(
        tags_update[0].value.to_string().trim(),
        "update_per_address_limit".to_string()
    );
    let tags_update_value = res
        .res
        .find_event_tags("wasm".to_string(), "limit".to_string());
    assert_eq!(
        tags_update_value[0].value.to_string().trim(),
        "5".to_string()
    );

    // Cannot mint more:
    let res = chain.orc.execute(
        "minter",
        "minter_exec_mint_token_err",
        &vending_minter::msg::ExecuteMsg::Mint {},
        &users[0],
        vec![OrcCoin {
            amount: MINT_PRICE,
            denom: denom.parse().unwrap(),
        }],
    );
    let err = res.unwrap_err();
    assert_matches!(err, ProcessError::CosmwasmError(TxError(..)));
    assert!(err
        .to_string()
        .contains("Max minting limit per address exceeded"));

    // Sleep to ensure we cannot mint after the end time
    chain
        .orc
        .poll_for_n_secs(100, Duration::from_millis(200_000))
        .unwrap();

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
    assert!(err.to_string().contains("Minting has ended"));

    // Purge -> ended
    let res = chain.orc.execute(
        "minter",
        "minter_exec_purge",
        &vending_minter::msg::ExecuteMsg::Purge {},
        &dev.key,
        vec![],
    );
    assert!(res.is_ok());
}
