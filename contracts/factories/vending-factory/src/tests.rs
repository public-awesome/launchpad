use crate::{
    msg::InstantiateMsg,
    state::{ParamsExtension, VendingMinterParams},
};
use cosmwasm_std::coin;
use sg_std::NATIVE_DENOM;

const CREATION_FEE: u128 = 5_000_000_000;
const MIN_MINT_PRICE: u128 = 50_000_000;
const MINT_FEE_FAIR_BURN: u64 = 1_000; // 10%
const MAX_TOKEN_LIMIT: u32 = 10000;
const MAX_PER_ADDRESS_LIMIT: u32 = 50;
const AIRDROP_MINT_PRICE: u128 = 0;
const AIRDROP_MINT_FEE_FAIR_BURN: u64 = 10_000; // 100%
const SHUFFLE_FEE: u128 = 500_000_000;

impl Default for VendingMinterParams {
    fn default() -> Self {
        Self {
            code_id: 1,
            allowed_sg721_code_ids: vec![1, 3, 5, 6],
            frozen: false,
            creation_fee: coin(CREATION_FEE, NATIVE_DENOM),
            min_mint_price: coin(MIN_MINT_PRICE, NATIVE_DENOM),
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
}

impl Default for InstantiateMsg {
    fn default() -> Self {
        Self {
            params: VendingMinterParams {
                code_id: 1,
                allowed_sg721_code_ids: vec![1, 3, 5, 6],
                frozen: false,
                creation_fee: coin(CREATION_FEE, NATIVE_DENOM),
                min_mint_price: coin(MIN_MINT_PRICE, NATIVE_DENOM),
                mint_fee_bps: MINT_FEE_FAIR_BURN,
                max_trading_offset_secs: 60 * 60 * 24 * 7,
                extension: ParamsExtension {
                    max_token_limit: MAX_TOKEN_LIMIT,
                    max_per_address_limit: MAX_PER_ADDRESS_LIMIT,
                    airdrop_mint_price: coin(AIRDROP_MINT_PRICE, NATIVE_DENOM),
                    airdrop_mint_fee_bps: AIRDROP_MINT_FEE_FAIR_BURN,
                    shuffle_fee: coin(SHUFFLE_FEE, NATIVE_DENOM),
                },
            },
        }
    }
}
