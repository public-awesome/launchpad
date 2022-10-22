use cosmwasm_schema::cw_serde;

use crate::MinterParams;

#[cw_serde]
pub enum Sg2QueryMsg {
    /// Returns `ParamsResponse`
    Params {},
}

#[cw_serde]
pub struct ParamsResponse<T> {
    pub params: MinterParams<T>,
}
