use base_factory::{msg::BaseMinterCreateMsg, state::BaseMinterParams};
use cosmwasm_std::coin;
use sg2::msg::CollectionParams;
use sg_std::NATIVE_DENOM;

const CREATION_FEE: u128 = 1_000_000_000;
pub const MIN_MINT_PRICE: u128 = 50_000_000;
const MINT_FEE_BPS: u64 = 10_000; // 100%

pub fn mock_params() -> BaseMinterParams {
    BaseMinterParams {
        code_id: 1,
        allowed_sg721_code_ids: vec![3],
        frozen: false,
        creation_fee: coin(CREATION_FEE, NATIVE_DENOM),
        min_mint_price: coin(MIN_MINT_PRICE, NATIVE_DENOM),
        mint_fee_bps: MINT_FEE_BPS,
        max_trading_offset_secs: 60 * 60 * 24 * 7,
        extension: None,
    }
}

pub fn mock_create_minter(collection_params: CollectionParams) -> BaseMinterCreateMsg {
    BaseMinterCreateMsg {
        init_msg: None,
        collection_params,
    }
}
