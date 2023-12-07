use assert_matches::assert_matches;
use cosm_orc::orchestrator::cosm_orc::tokio_block;
use cosm_orc::orchestrator::error::CosmwasmError::TxError;
use cosm_orc::orchestrator::error::ProcessError;
use cosm_orc::orchestrator::Coin as OrcCoin;
use cosm_orc::orchestrator::ExecReq;
use open_edition_factory::types::{NftData, NftMetadataType};
use sg2::msg::Sg2ExecuteMsg;
use sg_metadata::Metadata;
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
fn test_instantiate_open_edition_factory(chain: &mut Chain) {
    let creator = chain.cfg.users[0].clone();
    let dev = chain.cfg.users[1].clone();
    instantiate_factory(
        chain,
        creator.account.address,
        dev.account.address,
        &creator.key,
    )
    .unwrap();
}

#[test_context(Chain)]
#[test]
#[ignore]
fn test_create_minter(chain: &mut Chain) {
    let denom = chain.cfg.orc_cfg.chain_cfg.denom.clone();
    let user = chain.cfg.users[0].clone();
    let user_addr = &user.account.address;
    let dev = chain.cfg.users[1].clone();

    instantiate_factory(chain, user_addr.clone(), dev.account.address, &user.key).unwrap();

    let start_time = latest_block_time(chain).plus_seconds(10);
    let end_time = Some(latest_block_time(chain).plus_seconds(60));

    let valid_minter_msg = Sg2ExecuteMsg::CreateMinter(create_minter_msg(
        chain,
        None,
        user_addr.to_string(),
        MAX_TOKENS,
        start_time,
        end_time,
        Some(1_000u32),
        None,
        NftData {
            nft_data_type: NftMetadataType::OffChainMetadata,
            extension: None,
            token_uri: Some("ipfs://...".to_string()),
        },
    ));

    // requires fee
    let err = chain
        .orc
        .execute(
            FACTORY_NAME,
            "factory_exec_minter_inst_no_fee_err",
            &valid_minter_msg,
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
            &valid_minter_msg,
            &user.key,
            vec![OrcCoin {
                amount: 50_000_000,
                denom: denom.parse().unwrap(),
            }],
        )
        .unwrap_err();
    assert_matches!(err, ProcessError::CosmwasmError(TxError(..)));
    assert!(err.to_string().contains("Invalid Creation Fee amount"));

    // must be allowed collection
    let invalid_code_id_minter_msg = Sg2ExecuteMsg::CreateMinter(create_minter_msg(
        chain,
        Some(777u64),
        user_addr.to_string(),
        MAX_TOKENS,
        start_time,
        end_time,
        Some(1_000u32),
        None,
        NftData {
            nft_data_type: NftMetadataType::OffChainMetadata,
            extension: None,
            token_uri: Some("ipfs://...".to_string()),
        },
    ));
    let err = chain
        .orc
        .execute(
            FACTORY_NAME,
            "factory_exec_minter_inst_exact_fee_err",
            &invalid_code_id_minter_msg,
            &user.key,
            vec![OrcCoin {
                amount: CREATION_FEE,
                denom: denom.parse().unwrap(),
            }],
        )
        .unwrap_err();
    assert_matches!(err, ProcessError::CosmwasmError(TxError(..)));
    assert!(err.to_string().contains("InvalidCollectionCodeId"));

    // invalid nft data
    let invalid_nft_data_minter_msg = Sg2ExecuteMsg::CreateMinter(create_minter_msg(
        chain,
        None,
        user_addr.to_string(),
        MAX_TOKENS,
        start_time,
        end_time,
        Some(1_000u32),
        None,
        NftData {
            nft_data_type: NftMetadataType::OffChainMetadata,
            extension: Some(Metadata {
                image: None,
                image_data: None,
                external_url: None,
                description: None,
                name: None,
                attributes: None,
                background_color: None,
                animation_url: None,
                youtube_url: None,
            }),
            token_uri: Some("ipfs://...".to_string()),
        },
    ));
    let err = chain
        .orc
        .execute(
            FACTORY_NAME,
            "factory_exec_minter_inst_exact_fee_err",
            &invalid_nft_data_minter_msg,
            &user.key,
            vec![OrcCoin {
                amount: CREATION_FEE,
                denom: denom.parse().unwrap(),
            }],
        )
        .unwrap_err();
    assert_matches!(err, ProcessError::CosmwasmError(TxError(..)));
    assert!(err.to_string().contains("InvalidNftDataProvided"));

    // invalid nft data
    let invalid_nft_data_minter_msg = Sg2ExecuteMsg::CreateMinter(create_minter_msg(
        chain,
        None,
        user_addr.to_string(),
        MAX_TOKENS,
        start_time,
        end_time,
        Some(1_000u32),
        None,
        NftData {
            nft_data_type: NftMetadataType::OnChainMetadata,
            extension: None,
            token_uri: Some("ipfs://...".to_string()),
        },
    ));
    let err = chain
        .orc
        .execute(
            FACTORY_NAME,
            "factory_exec_minter_inst_exact_fee_err",
            &invalid_nft_data_minter_msg,
            &user.key,
            vec![OrcCoin {
                amount: CREATION_FEE,
                denom: denom.parse().unwrap(),
            }],
        )
        .unwrap_err();
    assert_matches!(err, ProcessError::CosmwasmError(TxError(..)));
    assert!(err.to_string().contains("InvalidNftDataProvided"));

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

    // Amount being burned
    let tags_burn = res
        .res
        .find_event_tags("burn".to_string(), "amount".to_string());
    assert!(!tags_burn[0].value.to_string().trim().is_empty());

    let tags = res
        .res
        .find_event_tags("instantiate".to_string(), "_contract_address".to_string());

    let (minter_addr, sg721_addr) = (tags[0].value.to_string(), tags[1].value.to_string());
    assert!(!minter_addr.trim().is_empty());
    assert!(!sg721_addr.trim().is_empty());

    // generate 10 user keys and send them all enough money to each mint 10 tokens (max)
    let users = gen_users(chain, 10, MINT_PRICE * MAX_TOKENS as u128 * 2u128, None);

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
        .poll_for_n_secs(10, Duration::from_millis(20_000))
        .unwrap();

    let mut total_mints = 0;
    let mut mints: HashMap<String, bool> = HashMap::new();

    // Batch mint all tokens:
    chain
        .orc
        .contract_map
        .add_address("minter", minter_addr)
        .unwrap();

    for user in &users {
        let mut reqs = vec![];
        for _ in 0..MAX_TOKENS {
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

    for n in 1..=MAX_TOKENS {
        assert_eq!(mints.get(&n.to_string()), Some(&true));
    }
    assert_eq!(total_mints, MAX_TOKENS * users.len() as u32);

    let balance = tokio_block(
        chain
            .orc
            .client
            .bank_query_balance(user_addr.parse().unwrap(), denom.parse().unwrap()),
    )
    .unwrap()
    .balance;

    // 10 * 10 * MINT_PRICE = 10k STARS x 0.9 (10% fee)
    assert_eq!(balance.amount, init_balance.amount + 9_000_000_000);

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
}

#[test_context(Chain)]
#[test]
#[ignore]
fn test_start_trading_time(chain: &mut Chain) {
    let denom = chain.cfg.orc_cfg.chain_cfg.denom.clone();
    let user = chain.cfg.users[0].clone();
    let user_addr = &user.account.address;
    let dev = chain.cfg.users[1].clone();

    let initial_total_supply =
        tokio_block(chain.orc.client.bank_query_supply(denom.parse().unwrap()))
            .unwrap()
            .balance;

    instantiate_factory(
        chain,
        user_addr.clone(),
        dev.account.address.clone(),
        &user.key,
    )
    .unwrap();

    let start_time = latest_block_time(chain).plus_seconds(5);
    let end_time = Some(latest_block_time(chain).plus_seconds(60));

    // The default offset is 1 day -> 86400
    let invalid_minter_msg = Sg2ExecuteMsg::CreateMinter(create_minter_msg(
        chain,
        None,
        user_addr.to_string(),
        MAX_TOKENS,
        start_time,
        end_time,
        Some(1_000u32),
        Some(start_time.plus_seconds(100_000)),
        NftData {
            nft_data_type: NftMetadataType::OffChainMetadata,
            extension: None,
            token_uri: Some("ipfs://...".to_string()),
        },
    ));

    let err = chain
        .orc
        .execute(
            FACTORY_NAME,
            "factory_exec_minter_inst_exact_fee_err",
            &invalid_minter_msg,
            &user.key,
            vec![OrcCoin {
                amount: CREATION_FEE,
                denom: denom.parse().unwrap(),
            }],
        )
        .unwrap_err();
    assert_matches!(err, ProcessError::CosmwasmError(TxError(..)));
    assert!(err.to_string().contains("InvalidStartTradingTime"));

    // Valid
    let minter_msg = Sg2ExecuteMsg::CreateMinter(create_minter_msg(
        chain,
        None,
        user_addr.to_string(),
        MAX_TOKENS,
        start_time,
        end_time,
        Some(1_000u32),
        Some(start_time.plus_seconds(80_400)),
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
    let mut total_dev_fees = 0;

    let fair_burn_fees = res
        .res
        .find_event_tags("fund_fairburn_pool".to_string(), "amount".to_string());

    let amount = fair_burn_fees[0].value.split(&denom).collect::<Vec<&str>>()[0];
    total_fairburn_fees += amount.parse::<u128>().unwrap();

    let users = gen_users(chain, 20, MINT_PRICE * 12, None);

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

    let balance_dev_before_mint = tokio_block(
        chain
            .orc
            .client
            .bank_query_balance(dev.account.address.parse().unwrap(), denom.parse().unwrap()),
    )
    .unwrap()
    .balance;

    // 20 users - 10 mints -> 200 mints
    for user in &users {
        let mut reqs = vec![];

        for _ in 0..10 {
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
        let initial_dev_balance = tokio_block(chain.orc.client.bank_query_balance(
            dev.account.address.clone().parse().unwrap(),
            denom.parse().unwrap(),
        ))
        .unwrap()
        .balance;
        let res = chain
            .orc
            .execute_batch("minter_batch_exec_mint_token_w_trading_time", reqs, user)
            .unwrap();
        let after_dev_balance = tokio_block(chain.orc.client.bank_query_balance(
            dev.account.address.clone().parse().unwrap(),
            denom.parse().unwrap(),
        ))
        .unwrap()
        .balance;
        // Because it is 10 mints and each mint, 50% of the mint fees goes to the dev
        // mint price = 100_000_000 * 0.1 * 0.5 = 5_000_000 * 10 = 50_000_000
        assert_eq!(
            after_dev_balance.amount - initial_dev_balance.amount,
            50_000_000
        );
        let fair_burn_fees = res
            .res
            .find_event_tags("fund_fairburn_pool".to_string(), "amount".to_string());

        for fee in fair_burn_fees {
            let amount = fee.value.split(&denom).collect::<Vec<&str>>()[0];
            total_fairburn_fees += amount.parse::<u128>().unwrap();
        }

        let dev_fees = res
            .res
            .find_event_tags("wasm-fair-burn".to_string(), "dev_amount".to_string());

        for fee in dev_fees {
            let amount = fee.value.split(&denom).collect::<Vec<&str>>()[0];
            total_dev_fees += amount.parse::<u128>().unwrap();
        }
    }

    // 200 mints at 100_000_000 * 0.1 * 0.5 = 1_000_000_000
    assert_eq!(total_dev_fees, 1_000_000_000);

    assert_eq!(total_mints, 200);

    let balance = tokio_block(
        chain
            .orc
            .client
            .bank_query_balance(user_addr.parse().unwrap(), denom.parse().unwrap()),
    )
    .unwrap()
    .balance;

    // 200 x MINT_PRICE = 10k STARS x 0.9 (10% fee)
    assert_eq!(balance.amount, init_balance.amount + 18_000_000_000);

    // fairburn fees
    // only the creation fee gets sent to the fairburn as 50%-50% = 0
    assert_eq!(total_fairburn_fees, 500_000_000);

    let total_supply = tokio_block(chain.orc.client.bank_query_supply(denom.parse().unwrap()))
        .unwrap()
        .balance;

    // 200 * 100_000_000 * 0.1 * 0.5 = 1_000_000_000
    let balance_dev_after_mint = tokio_block(
        chain
            .orc
            .client
            .bank_query_balance(dev.account.address.parse().unwrap(), denom.parse().unwrap()),
    )
    .unwrap()
    .balance;
    assert_eq!(
        balance_dev_after_mint.amount - balance_dev_before_mint.amount,
        1_000_000_000
    );

    // The amount of tokens burned should be
    // 500 STARS from the init + (200 mint x 100_000_000 x 0.1 x 0.5) -> 500 + 1_000 = 1_500
    assert_eq!(
        initial_total_supply.amount - 1_500_000_000,
        total_supply.amount
    );
}
