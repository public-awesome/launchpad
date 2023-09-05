use base_factory::{msg::BaseMinterCreateMsg, state::BaseMinterParams};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_binary, Binary, Empty, StdResult, Timestamp};
use sg4::MinterConfigResponse;

#[cw_serde]
pub struct InstantiateMsg {
    pub create_msg: BaseMinterCreateMsg,
    pub params: BaseMinterParams,
}

#[cw_serde]
pub enum ExecuteMsg {
    Mint { token_uri: String },
    UpdateStartTradingTime(Option<Timestamp>),
    AddPreMintHook { hook: String },
}

pub type ConfigResponse = MinterConfigResponse<Empty>;

#[cw_serde]
pub struct PreMintHookMsg {
    pub collection: String,
    pub token_id: Option<String>,
    pub buyer: String,
}

impl PreMintHookMsg {
    pub fn new(collection: String, token_id: Option<String>, buyer: String) -> Self {
        PreMintHookMsg {
            collection,
            token_id,
            buyer,
        }
    }

    /// serializes the message
    pub fn into_binary(self) -> StdResult<Binary> {
        let msg = PreMintExecuteHookMsg::PreMintHook(self);
        to_binary(&msg)
    }
}

// This is just a helper to properly serialize the above message
#[cw_serde]
pub enum PreMintExecuteHookMsg {
    PreMintHook(PreMintHookMsg),
}
