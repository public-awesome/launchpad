use cosmwasm_std::{coin, Coin, Timestamp, Uint128};
use open_edition_factory::types::NftData;
use open_edition_factory::{
    msg::{OpenEditionMinterCreateMsg, OpenEditionMinterInitMsgExtension},
    state::{OpenEditionMinterParams, ParamsExtension},
};
use sg2::msg::CollectionParams;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};

use crate::common_setup::msg::OpenEditionMinterCustomParams;
use crate::common_setup::setup_minter::common::constants::{
    CREATION_FEE, DEV_ADDRESS, MAX_TOKEN_LIMIT, MINT_FEE_FAIR_BURN, MIN_MINT_PRICE_OPEN_EDITION,
};

pub fn mock_init_minter_extension(
    start_time: Option<Timestamp>,
    end_time: Option<Timestamp>,
    per_address_limit_minter: Option<u32>,
    num_tokens: Option<u32>,
    mint_price: Option<Coin>,
    nft_data: NftData,
    payment_address: Option<String>,
) -> OpenEditionMinterInitMsgExtension {
    OpenEditionMinterInitMsgExtension {
        nft_data,
        start_time: start_time.unwrap_or(Timestamp::from_nanos(GENESIS_MINT_START_TIME)),
        mint_price: mint_price.unwrap_or_else(|| coin(MIN_MINT_PRICE_OPEN_EDITION, NATIVE_DENOM)),
        per_address_limit: per_address_limit_minter.unwrap_or(1u32),
        end_time,
        payment_address,
        num_tokens,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn mock_create_minter(
    start_time: Option<Timestamp>,
    end_time: Option<Timestamp>,
    mint_price: Option<Coin>,
    per_address_limit_minter: Option<u32>,
    num_tokens: Option<u32>,
    default_nft_data: NftData,
    collection_params: CollectionParams,
    payment_address: Option<String>,
) -> OpenEditionMinterCreateMsg {
    OpenEditionMinterCreateMsg {
        init_msg: mock_init_minter_extension(
            start_time,
            end_time,
            per_address_limit_minter,
            num_tokens,
            mint_price,
            default_nft_data,
            payment_address,
        ),
        collection_params,
    }
}

pub fn mock_create_minter_init_msg(
    collection_params: CollectionParams,
    init_msg: OpenEditionMinterInitMsgExtension,
) -> OpenEditionMinterCreateMsg {
    OpenEditionMinterCreateMsg {
        init_msg,
        collection_params,
    }
}

pub fn mock_params_proper() -> OpenEditionMinterParams {
    OpenEditionMinterParams {
        code_id: 1,
        allowed_sg721_code_ids: vec![1, 3, 5, 6],
        frozen: false,
        creation_fee: coin(CREATION_FEE, NATIVE_DENOM),
        min_mint_price: coin(MIN_MINT_PRICE_OPEN_EDITION, NATIVE_DENOM),
        mint_fee_bps: MINT_FEE_FAIR_BURN,
        max_trading_offset_secs: 60 * 60 * 24 * 7,
        extension: ParamsExtension {
            max_token_limit: MAX_TOKEN_LIMIT,
            max_per_address_limit: 10,
            airdrop_mint_fee_bps: 100,
            airdrop_mint_price: Coin {
                denom: NATIVE_DENOM.to_string(),
                amount: Uint128::new(100_000_000u128),
            },
            dev_fee_address: DEV_ADDRESS.to_string(),
        },
    }
}
// Pass custom params to change minter values
pub fn mock_params_custom(custom_params: OpenEditionMinterCustomParams) -> OpenEditionMinterParams {
    let denom = custom_params.denom.unwrap_or(NATIVE_DENOM);
    let mint_fee_bps = custom_params.mint_fee_bps.unwrap_or(MINT_FEE_FAIR_BURN);
    let airdrop_mint_price_amount = custom_params
        .airdrop_mint_price_amount
        .unwrap_or(Uint128::new(MIN_MINT_PRICE_OPEN_EDITION));
    OpenEditionMinterParams {
        code_id: 1,
        allowed_sg721_code_ids: vec![1, 3, 5, 6],
        frozen: false,
        creation_fee: coin(CREATION_FEE, NATIVE_DENOM),
        min_mint_price: coin(MIN_MINT_PRICE_OPEN_EDITION, denom),
        mint_fee_bps,
        max_trading_offset_secs: 60 * 60 * 24 * 7,
        extension: ParamsExtension {
            max_token_limit: MAX_TOKEN_LIMIT,
            max_per_address_limit: 10,
            airdrop_mint_fee_bps: 100,
            airdrop_mint_price: Coin {
                denom: denom.to_string(),
                amount: airdrop_mint_price_amount,
            },
            dev_fee_address: DEV_ADDRESS.to_string(),
        },
    }
}

pub fn mock_params_custom_min_mint_price(
    min_mint_price: Coin,
    airdrop_mint_price: Coin,
) -> OpenEditionMinterParams {
    OpenEditionMinterParams {
        code_id: 1,
        allowed_sg721_code_ids: vec![1, 3, 5, 6],
        frozen: false,
        creation_fee: coin(CREATION_FEE, NATIVE_DENOM),
        min_mint_price,
        mint_fee_bps: MINT_FEE_FAIR_BURN,
        max_trading_offset_secs: 60 * 60 * 24 * 7,
        extension: ParamsExtension {
            max_token_limit: MAX_TOKEN_LIMIT,
            max_per_address_limit: 10,
            airdrop_mint_fee_bps: 100,
            airdrop_mint_price,
            dev_fee_address: DEV_ADDRESS.to_string(),
        },
    }
}
