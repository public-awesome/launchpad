use base_factory::{msg::BaseMinterCreateMsg, state::BaseMinterParams};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Empty, Timestamp};
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
}

pub type ConfigResponse = MinterConfigResponse<Empty>;
