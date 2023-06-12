use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Timestamp};

use open_edition_factory::types::NftData;
use open_edition_factory::{msg::OpenEditionMinterCreateMsg, state::OpenEditionMinterParams};

#[cw_serde]
pub struct InstantiateMsg {
    pub create_msg: OpenEditionMinterCreateMsg,
    pub params: OpenEditionMinterParams,
}

#[cw_serde]
pub enum ExecuteMsg {
    Mint {},
    Purge {},
    /// Runs custom checks against TradingStartTime on VendingMinter, then updates by calling sg721-base
    UpdateStartTradingTime(Option<Timestamp>),
    UpdatePerAddressLimit {
        per_address_limit: u32,
    },
    MintTo {
        recipient: String,
    },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    StartTime {},
    MintPrice {},
    MintCount { address: String },
    Status {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub admin: String,
    pub nft_data: NftData,
    pub payment_address: Option<Addr>,
    pub per_address_limit: u32,
    pub end_time: Timestamp,
    pub sg721_address: String,
    pub sg721_code_id: u64,
    pub start_time: Timestamp,
    pub mint_price: Coin,
    pub factory: String,
}

#[cw_serde]
pub struct StartTimeResponse {
    pub start_time: String,
}

#[cw_serde]
pub struct MintPriceResponse {
    pub public_price: Coin,
    pub airdrop_price: Coin,
    pub current_price: Coin,
}

#[cw_serde]
pub struct MintCountResponse {
    pub address: String,
    pub count: u32,
}
