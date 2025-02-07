use crate::state::Stage;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, CosmosMsg, Empty, Timestamp};

#[cw_serde]
pub struct Member {
    pub address: String,
    pub mint_count: u32,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub stages: Vec<Stage>,
    pub merkle_roots: Vec<String>,
    pub merkle_tree_uris: Option<Vec<String>>,
    pub admins: Vec<String>,
    pub admins_mutable: bool,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateStageConfig(UpdateStageConfigMsg),
    UpdateAdmins { admins: Vec<String> },
    Freeze {},
}

#[cw_serde]
pub struct AdminListResponse {
    pub admins: Vec<String>,
    pub mutable: bool,
}

#[cw_serde]
pub struct UpdateStageConfigMsg {
    pub stage_id: u32,
    pub name: Option<String>,
    pub start_time: Option<Timestamp>,
    pub end_time: Option<Timestamp>,
    pub mint_price: Option<Coin>,
    pub per_address_limit: Option<u32>,
    pub mint_count_limit: Option<Option<u32>>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(HasStartedResponse)]
    HasStarted {},
    #[returns(HasEndedResponse)]
    HasEnded {},
    #[returns(IsActiveResponse)]
    IsActive {},
    #[returns(StageResponse)]
    ActiveStage {},
    #[returns(u32)]
    ActiveStageId {},
    #[returns(HasMemberResponse)]
    HasMember {
        member: String,
        proof_hashes: Vec<String>,
    },
    #[returns(ConfigResponse)]
    Config {},
    #[returns(StageResponse)]
    Stage { stage_id: u32 },
    #[returns(StagesResponse)]
    Stages {},
    #[returns(AdminListResponse)]
    AdminList {},
    #[returns(CanExecuteResponse)]
    CanExecute {
        sender: String,
        msg: CosmosMsg<Empty>,
    },
    #[returns(MerkleRootResponse)]
    MerkleRoots {},
    #[returns(MerkleTreeURIResponse)]
    MerkleTreeURIs {},
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
pub struct MerkleRootResponse {
    pub merkle_roots: Vec<String>,
}

#[cw_serde]
pub struct MerkleTreeURIResponse {
    pub merkle_tree_uris: Option<Vec<String>>,
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
    pub stage_id: u32,
    pub stage: Stage,
    pub merkle_root: String,
}

#[cw_serde]
pub struct StagesResponse {
    pub stages: Vec<StageResponse>,
}
