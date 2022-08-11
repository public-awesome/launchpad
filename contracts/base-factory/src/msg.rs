use cosmwasm_std::Empty;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sg2::msg::{CreateMinterMsg, Sg2ExecuteMsg, UpdateMinterParamsMsg};
use sg_std::StargazeMsgWrapper;

use crate::state::BaseMinterParams;

pub type Response = cosmwasm_std::Response<StargazeMsgWrapper>;
pub type SubMsg = cosmwasm_std::SubMsg<StargazeMsgWrapper>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub params: BaseMinterParams,
}

pub type BaseMinterCreateMsg = CreateMinterMsg<Empty>;

pub type ExecuteMsg = Sg2ExecuteMsg<Empty>;

pub type BaseUpdateParamsMsg = UpdateMinterParamsMsg<Empty>;

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// #[serde(rename_all = "snake_case")]
// pub enum SudoMsg {
//     UpdateParams(Box<BaseUpdateParamsMsg>),
//     UpdateMinterStatus {
//         minter: String,
//         verified: bool,
//         blocked: bool,
//     },
// }

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SudoMsg<T> {
    UpdateParams(Box<T>),
    UpdateMinterStatus {
        minter: String,
        verified: bool,
        blocked: bool,
    },
}
pub type BaseSudoMsg = SudoMsg<BaseUpdateParamsMsg>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ParamsResponse {
    pub params: BaseMinterParams,
}
