use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Timestamp};
use sg2::msg::{CreateMinterMsg, UpdateMinterParamsMsg};

use crate::state::VendingMinterParams;

#[cw_serde]
pub struct InstantiateMsg {
    pub params: VendingMinterParams,
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

#[cw_serde]
pub enum ExecuteMsg {
    CreateMinter(VendingMinterCreateMsg),
}

#[cw_serde]
pub enum SudoMsg {
    UpdateParams(Box<VendingUpdateParamsMsg>),
    AddContractToWhitelist {
        address: String,
    },
    RemoveContractFromWhitelist {
        address: String,
    },
    UpdateContractWhitelist {
        add: Vec<String>,
        remove: Vec<String>,
    },
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
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ParamsResponse)]
    Params {},
    #[returns(IsContractWhitelistedResponse)]
    IsContractWhitelisted { address: String },
    #[returns(WhitelistedContractsResponse)]
    WhitelistedContracts {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[cw_serde]
pub struct ParamsResponse {
    pub params: VendingMinterParams,
}

#[cw_serde]
pub struct IsContractWhitelistedResponse {
    pub address: String,
    pub is_whitelisted: bool,
}

#[cw_serde]
pub struct WhitelistedContractsResponse {
    pub contracts: Vec<String>,
}
