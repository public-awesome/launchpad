use cosmwasm_schema::cw_serde;

use crate::CodeId;
use crate::MinterParams;

#[cw_serde]
pub enum Sg2QueryMsg {
    /// Returns `ParamsResponse`
    Params {},
    AllowedCollectionCodeIds {},
    AllowedCollectionCodeId(CodeId),
    /// Returns `IsContractWhitelistedResponse`
    IsContractWhitelisted { address: String },
    /// Returns `WhitelistedContractsResponse`
    WhitelistedContracts {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[cw_serde]
pub struct ParamsResponse<T> {
    pub params: MinterParams<T>,
}

#[cw_serde]
pub struct AllowedCollectionCodeIdsResponse {
    pub code_ids: Vec<CodeId>,
}

#[cw_serde]
pub struct AllowedCollectionCodeIdResponse {
    pub allowed: bool,
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
