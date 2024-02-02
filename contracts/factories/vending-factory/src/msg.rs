use crate::state::VendingMinterParams;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, Timestamp};
use cw_vesting::vesting::Schedule;
use sg2::msg::{CreateMinterMsg, UpdateMinterParamsMsg};

#[cw_serde]
pub struct InstantiateMsg {
    pub params: VendingMinterParams,
}

#[cfg(test)]
impl Default for InstantiateMsg {
    fn default() -> Self {
        Self {
            params: VendingMinterParams {
                code_id: todo!(),
                allowed_sg721_code_ids: todo!(),
                frozen: todo!(),
                creation_fee: todo!(),
                min_mint_price: todo!(),
                mint_fee_bps: todo!(),
                max_trading_offset_secs: todo!(),
                extension: todo!(),
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

#[cw_serde]
pub struct TokenVaultVendingMinterInitMsgExtension {
    pub base: VendingMinterInitMsgExtension,
    pub vault_info: VaultInfo,
}
pub type TokenVaultVendingMinterCreateMsg =
    CreateMinterMsg<TokenVaultVendingMinterInitMsgExtension>;

// pub type ExecuteMsg = Sg2ExecuteMsg<VendingMinterInitMsgExtension>;

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
