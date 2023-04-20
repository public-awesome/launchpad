use cosmwasm_std::{coin, Coin, Timestamp, Uint128};
use sg2::msg::CollectionParams;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
use open_edition_factory::{
    msg::{OpenEditionMinterCreateMsg, OpenEditionMinterInitMsgExtension},
    state::{ParamsExtension, OpenEditionMinterParams},
};
use open_edition_factory::types::{NftData};

use crate::common_setup::setup_minter::common::constants::{CREATION_FEE, MINT_FEE_FAIR_BURN, DEV_ADDRESS, MIN_MINT_PRICE_OPEN_EDITION};

pub fn mock_init_minter_extension(
    start_time: Option<Timestamp>,
    end_time: Option<Timestamp>,
    per_address_limit_minter: Option<u32>,
    mint_price: Option<Coin>,
    nft_data: NftData,
    payment_address: Option<String>
) -> OpenEditionMinterInitMsgExtension {
    OpenEditionMinterInitMsgExtension {
        nft_data,
        start_time: start_time.unwrap_or(Timestamp::from_nanos(GENESIS_MINT_START_TIME)),
        mint_price: mint_price.unwrap_or(coin(MIN_MINT_PRICE_OPEN_EDITION, NATIVE_DENOM)),
        per_address_limit: per_address_limit_minter.unwrap_or(1u32),
        end_time: end_time.unwrap_or(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10_000)),
        payment_address,
    }
}

pub fn mock_create_minter(
    start_time: Option<Timestamp>,
    end_time: Option<Timestamp>,
    mint_price: Option<Coin>,
    per_address_limit_minter: Option<u32>,
    default_nft_data: NftData,
    collection_params: CollectionParams,
    payment_address: Option<String>
) -> OpenEditionMinterCreateMsg {
    OpenEditionMinterCreateMsg {
        init_msg: mock_init_minter_extension(start_time, end_time, per_address_limit_minter, mint_price, default_nft_data, payment_address),
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
            token_id_prefix_length: 30,
            abs_max_mint_per_address: 10,
            airdrop_mint_fee_bps: 100,
            airdrop_mint_price: Coin { denom: NATIVE_DENOM.to_string(), amount: Uint128::new(100_000_000u128) },
            dev_fee_address: DEV_ADDRESS.to_string(),
            dev_fee_bps: 200,
        },
    }
}
