use super::chain::Chain;
use cosm_orc::orchestrator::error::ProcessError;
use cosm_orc::orchestrator::{InstantiateResponse, SigningKey};
use cosmwasm_std::{Coin, Timestamp, Uint128};
use open_edition_factory::types::NftData;
use open_edition_factory::{
    msg::{InstantiateMsg, OpenEditionMinterInitMsgExtension},
    state::ParamsExtension,
};
use sg2::{
    msg::{CollectionParams, CreateMinterMsg},
    MinterParams,
};
use sg721::CollectionInfo;

// contract names used by cosm-orc to register stored code ids / instantiated addresses:
#[allow(dead_code)]
pub const SG721_NAME: &str = "sg721_base";
#[allow(dead_code)]
pub const FACTORY_NAME: &str = "open_edition_factory";
#[allow(dead_code)]
pub const MINTER_NAME: &str = "open_edition_minter";
#[allow(dead_code)]
pub const MAX_TOKENS: u32 = 10;
#[allow(dead_code)]
pub const CREATION_FEE: u128 = 1_000_000_000;
#[allow(dead_code)]
pub const MINT_PRICE: u128 = 100_000_000;

#[allow(dead_code)]
pub fn instantiate_factory(
    chain: &mut Chain,
    creator_addr: String,
    dev_addr: String,
    key: &SigningKey,
) -> Result<InstantiateResponse, ProcessError> {
    let denom = &chain.cfg.orc_cfg.chain_cfg.denom;

    chain.orc.instantiate(
        FACTORY_NAME,
        "factory_inst",
        &InstantiateMsg {
            params: MinterParams {
                code_id: chain.orc.contract_map.code_id(MINTER_NAME).unwrap(),
                allowed_sg721_code_ids: vec![chain.orc.contract_map.code_id(SG721_NAME).unwrap()],
                frozen: false,
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
                    max_token_limit: 1_000u32,
                    max_per_address_limit: 50,
                    airdrop_mint_fee_bps: 0,
                    airdrop_mint_price: Coin {
                        amount: Uint128::new(0),
                        denom: denom.to_string(),
                    },
                    dev_fee_address: dev_addr,
                },
            },
        },
        key,
        Some(creator_addr.parse().unwrap()),
        vec![],
    )
}

#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
pub fn create_minter_msg(
    chain: &mut Chain,
    code_id: Option<u64>,
    creator_addr: String,
    limit: u32,
    start_time: Timestamp,
    end_time: Option<Timestamp>,
    num_tokens: Option<u32>,
    start_trading_time: Option<Timestamp>,
    nft_data: NftData,
) -> CreateMinterMsg<OpenEditionMinterInitMsgExtension> {
    let denom = &chain.cfg.orc_cfg.chain_cfg.denom;

    CreateMinterMsg {
        init_msg: OpenEditionMinterInitMsgExtension {
            nft_data,
            start_time,
            payment_address: Some(creator_addr.clone()),
            mint_price: Coin {
                amount: Uint128::new(MINT_PRICE),
                denom: denom.to_string(),
            },
            per_address_limit: limit,
            end_time,
            num_tokens,
        },
        collection_params: CollectionParams {
            code_id: code_id.unwrap_or_else(|| chain.orc.contract_map.code_id(SG721_NAME).unwrap()),
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
