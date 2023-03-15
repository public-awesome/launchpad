use cosmwasm_schema::cw_serde;

use crate::CodeId;
use crate::MinterParams;

#[cw_serde]
pub enum Sg2QueryMsg {
    /// Returns `ParamsResponse`
    Params {},
    AllowedCollectionCodeIds {},
    AllowedCollectionCodeId(CodeId),
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
