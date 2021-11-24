use crate::state::{CreatorInfo, Extension};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
  pub name: String,
  pub symbol: String,
  pub minter: String,
  pub creator_info: CreatorInfo,
}

// specialize ExecuteMsg with the CreatorInfo extention
pub type ExecuteMsg = cw721_base::ExecuteMsg<Extension>;

pub type QueryMsg = cw721_base::QueryMsg;
