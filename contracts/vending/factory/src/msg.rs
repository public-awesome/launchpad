use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sg2_vending::{VendingMinterParams, VendingUpdateParamsMsg};
use sg_std::StargazeMsgWrapper;

pub type Response = cosmwasm_std::Response<StargazeMsgWrapper>;
pub type SubMsg = cosmwasm_std::SubMsg<StargazeMsgWrapper>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub params: VendingMinterParams,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SudoMsg {
    UpdateParams(Box<VendingUpdateParamsMsg>),
    UpdateMinterStatus {
        minter: String,
        verified: bool,
        blocked: bool,
    },
}
