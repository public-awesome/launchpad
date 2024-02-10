use crate::state::{ParamsExtension, VendingMinterParams};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{coin, Coin, Timestamp};
use cw_vesting::vesting::Schedule;
use sg2::msg::{CreateMinterMsg, UpdateMinterParamsMsg};
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};

#[cw_serde]
pub struct InstantiateMsg {
    pub params: VendingMinterParams,
}

// TODO: find a way to export this for tests only (in other crates)
impl Default for InstantiateMsg {
    fn default() -> Self {
        Self {
            params: VendingMinterParams {
                code_id: 1,
                allowed_sg721_code_ids: vec![1, 3, 5, 6],
                frozen: false,
                creation_fee: coin(5_000_000_000, NATIVE_DENOM),
                min_mint_price: coin(50_000_000, NATIVE_DENOM),
                mint_fee_bps: 1_000,
                max_trading_offset_secs: 60 * 60 * 24 * 7,
                extension: ParamsExtension {
                    max_token_limit: 10000,
                    max_per_address_limit: 50,
                    airdrop_mint_price: coin(0, NATIVE_DENOM),
                    airdrop_mint_fee_bps: 10_000,
                    shuffle_fee: coin(500_000_000, NATIVE_DENOM),
                },
            },
        }
    }
}

#[cw_serde]
pub struct VendingMinterInitMsgExtension {
    pub base_token_uri: String,
    pub payment_address: Option<String>,
    pub start_time: Timestamp,
    pub num_tokens: u32,
    pub mint_price: Coin,
    pub per_address_limit: u32,
    pub whitelist: Option<String>,
}

// TODO: find a way to export this for tests only (in other crates)
impl Default for VendingMinterInitMsgExtension {
    fn default() -> Self {
        Self {
            base_token_uri: "ipfs://aldkfjads".to_string(),
            payment_address: None,
            start_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME),
            num_tokens: 100,
            mint_price: coin(100_000_000, NATIVE_DENOM),
            per_address_limit: 3,
            whitelist: None,
        }
    }
}

pub type VendingMinterCreateMsg = CreateMinterMsg<VendingMinterInitMsgExtension>;

// vesting_duration_seconds: 3 * 365 * 24 * 60 * 60,
// unbonding_duration_seconds: 14 * 24 * 60 * 60,
#[cw_serde]
pub struct VaultInfo {
    pub token_balance: Coin,
    pub vesting_schedule: Schedule,
    pub vesting_duration_seconds: u64,
    pub unbonding_duration_seconds: u64,
    pub vesting_code_id: u64,
}

// TODO: find a way to export this for tests only (in other crates)
impl Default for VaultInfo {
    fn default() -> Self {
        Self {
            token_balance: coin(500000u128, NATIVE_DENOM),
            vesting_schedule: cw_vesting::vesting::Schedule::SaturatingLinear,
            vesting_duration_seconds: 3 * 365 * 24 * 60 * 60,
            unbonding_duration_seconds: 14 * 24 * 60 * 60,
            vesting_code_id: 1,
        }
    }
}

#[cw_serde]
pub struct TokenVaultVendingMinterInitMsgExtension {
    pub base: VendingMinterInitMsgExtension,
    pub vault_info: VaultInfo,
}
pub type TokenVaultVendingMinterCreateMsg =
    CreateMinterMsg<TokenVaultVendingMinterInitMsgExtension>;

#[cw_serde]
pub enum ExecuteMsg {
    CreateMinter(VendingMinterCreateMsg),
    CreateTokenVaultMinter(TokenVaultVendingMinterCreateMsg),
}

#[cw_serde]
pub enum SudoMsg {
    UpdateParams(Box<VendingUpdateParamsMsg>),
}

/// Message for params so they can be updated individually by governance
#[cw_serde]
pub struct VendingUpdateParamsExtension {
    pub max_token_limit: Option<u32>,
    pub max_per_address_limit: Option<u32>,
    pub airdrop_mint_price: Option<Coin>,
    pub airdrop_mint_fee_bps: Option<u64>,
    pub shuffle_fee: Option<Coin>,
}
pub type VendingUpdateParamsMsg = UpdateMinterParamsMsg<VendingUpdateParamsExtension>;

#[cw_serde]
pub struct ParamsResponse {
    pub params: VendingMinterParams,
}
