use cosmwasm_schema::cw_serde;
use cosmwasm_std::Empty;
use sg2::msg::{CreateMinterMsg, Sg2ExecuteMsg, UpdateMinterParamsMsg};

use crate::state::BaseMinterParams;

#[cw_serde]
pub struct InstantiateMsg {
    pub params: BaseMinterParams,
}

pub type Extension = Option<Empty>;

pub type BaseMinterCreateMsg = CreateMinterMsg<Extension>;

pub type ExecuteMsg = Sg2ExecuteMsg<Extension>;

pub type BaseUpdateParamsMsg = UpdateMinterParamsMsg<Extension>;

#[cw_serde]
pub enum SudoMsg<T> {
    UpdateParams(Box<T>),
}
pub type BaseSudoMsg = SudoMsg<BaseUpdateParamsMsg>;

#[cw_serde]
pub struct ParamsResponse {
    pub params: BaseMinterParams,
}
