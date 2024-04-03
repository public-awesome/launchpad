use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, CosmosMsg, Empty, Timestamp};

#[cw_serde]
pub struct Member {
    pub address: String,
    pub mint_count: u32,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub merkle_root: String,
    pub merkle_tree_uri: Option<String>,

    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub mint_price: Coin,
    pub per_address_limit: u32,

    pub admins: Vec<String>,
    pub admins_mutable: bool,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateStartTime(Timestamp),
    UpdateEndTime(Timestamp),
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
    pub to_add: Vec<Member>,
}

#[cw_serde]
pub struct RemoveMembersMsg {
    pub to_remove: Vec<String>,
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
    #[returns(HasMemberResponse)]
    HasMember {
        member: String,
        proof_hashes: Vec<String>,
    },
    #[returns(ConfigResponse)]
    Config {},
    #[returns(AdminListResponse)]
    AdminList {},
    #[returns(CanExecuteResponse)]
    CanExecute {
        sender: String,
        msg: CosmosMsg<Empty>,
    },
    #[returns(MerkleRootResponse)]
    MerkleRoot {},
    #[returns(MerkleTreeURIResponse)]
    MerkleTreeURI {},
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
    pub merkle_root: String,
}

#[cw_serde]
pub struct MerkleTreeURIResponse {
    pub merkle_tree_uri: Option<String>,
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
