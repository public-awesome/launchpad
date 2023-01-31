use super::chain::Chain;
use cosm_orc::orchestrator::cosm_orc::tokio_block;
use cosm_orc::orchestrator::error::ProcessError;
use cosm_orc::orchestrator::Coin as OrcCoin;
use cosm_orc::orchestrator::{InstantiateResponse, SigningKey};
use cosm_tome::chain::request::TxOptions;
use cosm_tome::modules::bank::model::SendRequest;
use cosmwasm_std::{Coin, Timestamp, Uint128};
use sg2::{
    msg::{CollectionParams, CreateMinterMsg},
    MinterParams,
};
use sg721::CollectionInfo;
use vending_factory::{
    msg::{InstantiateMsg, VendingMinterInitMsgExtension},
    state::ParamsExtension,
};

// contract names used by cosm-orc to register stored code ids / instantiated addresses:
pub const SG721_NAME: &str = "sg721_base";
pub const FACTORY_NAME: &str = "vending_factory";
pub const MINTER_NAME: &str = "vending_minter";

pub const MAX_TOKENS: u32 = 10_000;
pub const CREATION_FEE: u128 = 1_000_000_000;
pub const MINT_PRICE: u128 = 100_000_000;

pub fn instantiate_factory(
    chain: &mut Chain,
    creator_addr: String,
    key: &SigningKey,
) -> Result<InstantiateResponse, ProcessError> {
    let denom = &chain.cfg.orc_cfg.chain_cfg.denom;

    chain.orc.instantiate(
        FACTORY_NAME,
        "factory_inst",
        &InstantiateMsg {
            params: MinterParams {
                code_id: chain.orc.contract_map.code_id(MINTER_NAME).unwrap(),
                creation_fee: Coin {
                    amount: Uint128::new(CREATION_FEE),
                    denom: denom.to_string(),
                },
                min_mint_price: Coin {
                    amount: Uint128::new(50),
                    denom: denom.to_string(),
                },
                mint_fee_bps: 1000, // 10%
                max_trading_offset_secs: (60 * 60) * 24,
                extension: ParamsExtension {
                    max_token_limit: MAX_TOKENS,
                    max_per_address_limit: 50,
                    airdrop_mint_fee_bps: 0,
                    airdrop_mint_price: Coin {
                        amount: Uint128::new(0),
                        denom: denom.to_string(),
                    },
                    shuffle_fee: Coin {
                        amount: Uint128::new(500_000_000),
                        denom: denom.to_string(),
                    },
                },
            },
        },
        key,
        Some(creator_addr.parse().unwrap()),
        vec![],
    )
}

pub fn create_minter_msg(
    chain: &mut Chain,
    creator_addr: String,
    num_tokens: u32,
    limit: u32,
    start_time: Timestamp,
    start_trading_time: Option<Timestamp>,
) -> CreateMinterMsg<VendingMinterInitMsgExtension> {
    let denom = &chain.cfg.orc_cfg.chain_cfg.denom;

    CreateMinterMsg {
        init_msg: VendingMinterInitMsgExtension {
            base_token_uri: "ipfs://...".to_string(),
            payment_address: Some(creator_addr.clone()),
            start_time,
            num_tokens,
            mint_price: Coin {
                amount: Uint128::new(MINT_PRICE),
                denom: denom.to_string(),
            },
            per_address_limit: limit,
            whitelist: None,
        },
        collection_params: CollectionParams {
            code_id: chain.orc.contract_map.code_id(SG721_NAME).unwrap(),
            name: "Collection".to_string(),
            symbol: "SYM".to_string(),
            info: CollectionInfo {
                creator: creator_addr,
                description: "Description".to_string(),
                image: "https://example.com/image.png".to_string(),
                start_trading_time,
                external_link: None,
                explicit_content: None,
                royalty_info: None,
            },
        },
    }
}

// gen_users will create `num_users` random SigningKeys
// and then transfer `init_balance` of funds to each of them.
pub fn gen_users(chain: &mut Chain, num_users: u32, init_balance: u128) -> Vec<SigningKey> {
    let prefix = &chain.cfg.orc_cfg.chain_cfg.prefix;
    let denom = &chain.cfg.orc_cfg.chain_cfg.denom;
    let from_user = &chain.cfg.users[1];

    let mut users = vec![];
    for n in 0..num_users {
        users.push(SigningKey::random_mnemonic(n.to_string()));
    }

    let mut reqs = vec![];
    for user in &users {
        reqs.push(SendRequest {
            from: from_user.account.address.parse().unwrap(),
            to: user.to_addr(prefix).unwrap(),
            amounts: vec![OrcCoin {
                amount: init_balance,
                denom: denom.parse().unwrap(),
            }],
        });
    }

    tokio_block(
        chain
            .orc
            .client
            .bank_send_batch(reqs, &from_user.key, &TxOptions::default()),
    )
    .unwrap();

    users
}

pub fn latest_block_time(chain: &Chain) -> Timestamp {
    let now = tokio_block(chain.orc.client.tendermint_query_latest_block())
        .unwrap()
        .block
        .header
        .unwrap()
        .time
        .unwrap();

    Timestamp::from_seconds(now.seconds.try_into().unwrap())
}
