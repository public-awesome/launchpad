use cosmwasm_schema::cw_serde;

use crate::CodeId;
use crate::MinterParams;

#[cw_serde]
pub enum Sg2QueryMsg {
    /// Returns `ParamsResponse`
    Params {},
    /// Returns allowed sg721 code ids
    AllowedSg721CodeIds {},
}

#[cw_serde]
pub struct ParamsResponse<T> {
    pub params: MinterParams<T>,
}

#[cw_serde]
pub struct Sg721CodeIdsResponse {
    pub code_ids: Vec<CodeId>,
}
