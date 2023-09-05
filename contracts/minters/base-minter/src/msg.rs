use base_factory::{msg::BaseMinterCreateMsg, state::BaseMinterParams};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Empty, Timestamp};
use sg4::{MinterConfigResponse, StatusResponse};
use sg_mint_hooks::{sg_mint_hooks_execute, sg_mint_hooks_query};

#[cw_serde]
pub struct InstantiateMsg {
    pub create_msg: BaseMinterCreateMsg,
    pub params: BaseMinterParams,
}

#[sg_mint_hooks_execute]
#[cw_serde]
pub enum ExecuteMsg {
    Mint { token_uri: String },
    UpdateStartTradingTime(Option<Timestamp>),
}

pub type ConfigResponse = MinterConfigResponse<Empty>;

#[sg_mint_hooks_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(StatusResponse)]
    Status {},
}
