use crate::state::Creator;

pub type InstantiateMsg = cw721_base::InstantiateMsg;

// specialize ExecuteMsg with the Creator extention
pub type Extension = Creator;
pub type ExecuteMsg = cw721_base::ExecuteMsg<Extension>;

pub type QueryMsg = cw721_base::QueryMsg;
