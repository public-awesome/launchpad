use crate::state::Stage;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, CosmosMsg, Empty, Timestamp};

#[cw_serde]
pub struct InstantiateMsg {
    pub members: Vec<Vec<String>>,
    pub stages: Vec<Stage>,
    pub member_limit: u32,
    pub admins: Vec<String>,
    pub admins_mutable: bool,
}

#[cw_serde]
pub enum ExecuteMsg {
    AddStage(AddStageMsg),
    RemoveStage(RemoveStageMsg),
    AddMembers(AddMembersMsg),
    RemoveMembers(RemoveMembersMsg),
    UpdateStageConfig(UpdateStageConfigMsg),
    IncreaseMemberLimit(u32),
    UpdateAdmins { admins: Vec<String> },
    Freeze {},
}

#[cw_serde]
pub struct AdminListResponse {
    pub admins: Vec<String>,
    pub mutable: bool,
}

#[cw_serde]
pub struct AddMembersMsg {
    pub to_add: Vec<String>,
    pub stage_id: u32,
}

#[cw_serde]
pub struct RemoveMembersMsg {
    pub to_remove: Vec<String>,
    pub stage_id: u32,
}
#[cw_serde]
pub struct AddStageMsg {
    pub stage: Stage,
    pub members: Vec<String>,
}

#[cw_serde]
pub struct RemoveStageMsg {
    pub stage_id: u32,
}

#[cw_serde]
pub struct UpdateStageConfigMsg {
    pub stage_id: u32,
    pub name: Option<String>,
    pub start_time: Option<Timestamp>,
    pub end_time: Option<Timestamp>,
    pub mint_price: Option<Coin>,
    pub per_address_limit: Option<u32>,
}

#[cw_serde]
pub enum QueryMsg {
    HasStarted {},
    HasEnded {},
    IsActive {},
    ActiveStage {},
    ActiveStageId {},
    Members {
        start_after: Option<String>,
        limit: Option<u32>,
        stage_id: u32,
    },
    HasMember {
        member: String,
    },
    Config {},

    Stage {
        stage_id: u32,
    },

    Stages {},

    AdminList {},

    CanExecute {
        sender: String,
        msg: CosmosMsg<Empty>,
    },
}

#[cw_serde]
pub struct MembersResponse {
    pub members: Vec<String>,
}

#[cw_serde]
pub struct HasMemberResponse {
    pub has_member: bool,
}

#[cw_serde]
pub struct HasEndedResponse {
    pub has_ended: bool,
}

#[cw_serde]
pub struct HasStartedResponse {
    pub has_started: bool,
}

#[cw_serde]
pub struct IsActiveResponse {
    pub is_active: bool,
}

#[cw_serde]
pub struct MintPriceResponse {
    pub mint_price: Coin,
}

#[cw_serde]
pub struct ConfigResponse {
    pub num_members: u32,
    pub per_address_limit: u32,
    pub member_limit: u32,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub mint_price: Coin,
    pub is_active: bool,
}

#[cw_serde]
pub enum SudoMsg {
    /// Add a new operator
    AddOperator { operator: String },
    /// Remove operator
    RemoveOperator { operator: String },
}

#[cw_serde]
pub struct CanExecuteResponse {
    pub can_execute: bool,
}

#[cw_serde]
pub struct StageResponse {
    pub stage: Stage,
}

#[cw_serde]
pub struct StagesResponse {
    pub stages: Vec<Stage>,
}
