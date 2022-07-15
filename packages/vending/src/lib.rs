use cosmwasm_std::{Coin, Timestamp};
use minters::{CreateMinterMsg, MinterParams};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Parameters common to all vending minters, as determined by governance
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ParamsExtension {
    pub shuffle_fee: Coin,
}
pub type VendingMinterParams = MinterParams<ParamsExtension>;

/// Properties of each specific vending minter
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MinterInitMsgExtension {
    pub base_token_uri: String,
    pub start_time: Timestamp,
    pub num_tokens: u32,
    pub unit_price: Coin,
    pub per_address_limit: u32,
    pub whitelist: Option<String>,
}
pub type VendingMinterCreateMsg = CreateMinterMsg<MinterInitMsgExtension>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Params {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ParamsResponse {
    pub params: VendingMinterParams,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreateVendingMinter(VendingMinterCreateMsg),
}

pub mod tests {
    use cosmwasm_std::{coin, Timestamp};
    use minters::tests::mock_collection_params;
    use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};

    use crate::{
        MinterInitMsgExtension, ParamsExtension, VendingMinterCreateMsg, VendingMinterParams,
    };

    pub const CREATION_FEE: u128 = 5_000_000_000;
    pub const MIN_MINT_PRICE: u128 = 50_000_000;
    pub const AIRDROP_MINT_PRICE: u128 = 15_000_000;
    pub const MINT_FEE_BPS: u64 = 1_000; // 10%
    pub const AIRDROP_MINT_FEE_BPS: u64 = 10_000; // 100%
    pub const SHUFFLE_FEE: u128 = 500_000_000;
    pub const MAX_TOKEN_LIMIT: u32 = 10_000;
    pub const MAX_PER_ADDRESS_LIMIT: u32 = 50;

    pub fn mock_init_extension() -> MinterInitMsgExtension {
        MinterInitMsgExtension {
            base_token_uri: "ipfs://aldkfjads".to_string(),
            start_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME),
            num_tokens: 100,
            unit_price: coin(MIN_MINT_PRICE, NATIVE_DENOM),
            per_address_limit: 5,
            whitelist: None,
        }
    }

    pub fn mock_params() -> VendingMinterParams {
        VendingMinterParams {
            code_id: 1,
            creation_fee: coin(CREATION_FEE, NATIVE_DENOM),
            max_token_limit: MAX_TOKEN_LIMIT,
            max_per_address_limit: MAX_PER_ADDRESS_LIMIT,
            min_mint_price: coin(MIN_MINT_PRICE, NATIVE_DENOM),
            airdrop_mint_price: coin(AIRDROP_MINT_PRICE, NATIVE_DENOM),
            mint_fee_bps: MINT_FEE_BPS,
            airdrop_mint_fee_bps: AIRDROP_MINT_FEE_BPS,
            extension: ParamsExtension {
                shuffle_fee: coin(SHUFFLE_FEE, NATIVE_DENOM),
            },
        }
    }

    pub fn mock_create_minter() -> VendingMinterCreateMsg {
        VendingMinterCreateMsg {
            init_msg: mock_init_extension(),
            collection_params: mock_collection_params(),
        }
    }
}
