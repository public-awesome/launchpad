use cosmwasm_schema::cw_serde;
use cosmwasm_std::Timestamp;
use cw721::Cw721ReceiveMsg;
use token_merge_factory::msg::MintToken;

#[cw_serde]
pub enum ExecuteMsg {
    ReceiveNft(Cw721ReceiveMsg),
    Purge {},
    UpdateStartTime(Timestamp),
    /// Runs custom checks against TradingStartTime on TokenMergeMinter, then updates by calling sg721-base
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
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    MintableNumTokens {},
    StartTime {},
    MintTokens {},
    MintCount { address: String },
    DepositedTokens { address: String },
    Status {},
}

#[cw_serde]
pub enum ReceiveNftMsg {
    DepositToken { recipient: Option<String> },
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
    pub mint_tokens: Vec<MintToken>,
    pub factory: String,
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
pub struct MintTokensResponse {
    pub mint_tokens: Vec<MintToken>,
}

#[cw_serde]
pub struct MintCountResponse {
    pub address: String,
    pub count: u32,
}
