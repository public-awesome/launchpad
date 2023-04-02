use cosmwasm_std::{coin, Timestamp};
use sg2::msg::CollectionParams;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
use vending_factory::{
    msg::{VendingMinterCreateMsg, VendingMinterInitMsgExtension},
    state::{ParamsExtension, VendingMinterParams},
};

use crate::common_setup::setup_minter::common::constants::{
    AIRDROP_MINT_FEE_FAIR_BURN, AIRDROP_MINT_PRICE, CREATION_FEE, MAX_PER_ADDRESS_LIMIT,
    MAX_TOKEN_LIMIT, MINT_FEE_FAIR_BURN, MIN_MINT_PRICE, SHUFFLE_FEE,
};

pub fn mock_init_extension(
    splits_addr: Option<String>,
    start_time: Option<Timestamp>,
) -> VendingMinterInitMsgExtension {
    vending_factory::msg::VendingMinterInitMsgExtension {
        base_token_uri: "ipfs://aldkfjads".to_string(),
        payment_address: splits_addr,
        start_time: start_time.unwrap_or(Timestamp::from_nanos(GENESIS_MINT_START_TIME)),
        num_tokens: 100,
        mint_price: coin(MIN_MINT_PRICE, NATIVE_DENOM),
        per_address_limit: 3,
        whitelist: None,
    }
}

pub fn mock_create_minter(
    splits_addr: Option<String>,
    collection_params: CollectionParams,
    start_time: Option<Timestamp>,
) -> VendingMinterCreateMsg {
    VendingMinterCreateMsg {
        init_msg: mock_init_extension(splits_addr, start_time),
        collection_params,
    }
}

pub fn mock_create_minter_init_msg(
    collection_params: CollectionParams,
    init_msg: VendingMinterInitMsgExtension,
) -> VendingMinterCreateMsg {
    VendingMinterCreateMsg {
        init_msg,
        collection_params,
    }
}

pub fn mock_params(mint_denom: Option<String>) -> VendingMinterParams {
    VendingMinterParams {
        code_id: 1,
        allowed_sg721_code_ids: vec![1, 3, 5, 6],
        frozen: false,
        creation_fee: coin(CREATION_FEE, NATIVE_DENOM),
        min_mint_price: coin(
            MIN_MINT_PRICE,
            mint_denom.unwrap_or_else(|| NATIVE_DENOM.to_string()),
        ),
        mint_fee_bps: MINT_FEE_FAIR_BURN,
        max_trading_offset_secs: 60 * 60 * 24 * 7,
        extension: ParamsExtension {
            max_token_limit: MAX_TOKEN_LIMIT,
            max_per_address_limit: MAX_PER_ADDRESS_LIMIT,
            airdrop_mint_price: coin(AIRDROP_MINT_PRICE, NATIVE_DENOM),
            airdrop_mint_fee_bps: AIRDROP_MINT_FEE_FAIR_BURN,
            shuffle_fee: coin(SHUFFLE_FEE, NATIVE_DENOM),
        },
    }
}
