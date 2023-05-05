use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Timestamp};
use sg4::StatusResponse;
use vending_factory::{msg::VendingMinterCreateMsg, state::VendingMinterParams};

#[cw_serde]
pub struct InstantiateMsg {
    pub create_msg: VendingMinterCreateMsg,
    pub params: VendingMinterParams,
}

#[cw_serde]
pub enum ExecuteMsg {
    Mint {},
    SetWhitelist {
        whitelist: String,
    },
    Purge {},
    UpdateMintPrice {
        price: u128,
    },
    UpdateStartTime(Timestamp),
    /// Runs custom checks against TradingStartTime on VendingMinter, then updates by calling sg721-base
    UpdateStartTradingTime(Option<Timestamp>),
    UpdatePerAddressLimit {
        per_address_limit: u32,
    },
    MintTo {
        recipient: String,
    },
    MintFor {
        token_id: u32,
        recipient: String,
    },
    Shuffle {},
    BurnRemaining {},
    UpdateDiscountPrice {
        price: u128,
    },
    RemoveDiscountPrice {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(MintableNumTokensResponse)]
    MintableNumTokens {},
    #[returns(StartTimeResponse)]
    StartTime {},
    #[returns(MintPriceResponse)]
    MintPrice {},
    #[returns(MintCountResponse)]
    MintCount { address: String },
    #[returns(StatusResponse)]
    Status {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub admin: String,
    pub base_token_uri: String,
    pub num_tokens: u32,
    pub per_address_limit: u32,
    pub sg721_address: String,
    pub sg721_code_id: u64,
    pub start_time: Timestamp,
    pub mint_price: Coin,
    pub whitelist: Option<String>,
    pub factory: String,
    pub discount_price: Option<Coin>,
}

#[cw_serde]
pub struct MintableNumTokensResponse {
    pub count: u32,
}

#[cw_serde]
pub struct StartTimeResponse {
    pub start_time: String,
}

#[cw_serde]
pub struct MintPriceResponse {
    pub public_price: Coin,
    pub airdrop_price: Coin,
    pub whitelist_price: Option<Coin>,
    pub current_price: Coin,
    pub discount_price: Option<Coin>,
}

#[cw_serde]
pub struct MintCountResponse {
    pub address: String,
    pub count: u32,
}
