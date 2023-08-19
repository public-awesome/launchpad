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
    open_edition_ibc_minter_helpers::{
        create_minter_msg, instantiate_factory, CREATION_FEE, FACTORY_NAME, MAX_TOKENS, MINT_DENOM,
        MINT_PRICE,
    },
};

#[test_context(Chain)]
#[test]
#[ignore]
fn test_create_open_edition_ibc_minter(chain: &mut Chain) {
    let denom = chain.cfg.orc_cfg.chain_cfg.denom.clone();
    let user = chain.cfg.users[0].clone();
    let user_addr = &user.account.address;
    let dev = chain.cfg.users[1].clone();

    instantiate_factory(chain, user_addr.clone(), dev.account.address, &user.key).unwrap();

    let start_time = latest_block_time(chain).plus_seconds(10);
    let end_time = latest_block_time(chain).plus_seconds(60);

    let valid_minter_msg = Sg2ExecuteMsg::CreateMinter(create_minter_msg(
        chain,
        None,
        user_addr.to_string(),
        MAX_TOKENS,
        start_time,
        end_time,
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
    let users = gen_users(
        chain,
        10,
        MINT_PRICE * MAX_TOKENS as u128 * 2u128,
        Some(MINT_DENOM),
    );

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
                    denom: MINT_DENOM.parse().unwrap(),
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
            .bank_query_balance(user_addr.parse().unwrap(), MINT_DENOM.parse().unwrap()),
    )
    .unwrap()
    .balance;

    // 10 * 10 * MINT_PRICE = 10k STARS x 0 (100% fee)
    assert_eq!(balance.amount, init_balance.amount);

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
            denom: MINT_DENOM.parse().unwrap(),
        }],
    );

    let err = res.unwrap_err();
    assert_matches!(err, ProcessError::CosmwasmError(TxError(..)));
    assert!(err.to_string().contains("Minting has ended"));
}
