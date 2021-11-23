use crate::state::Extension;

pub type InstantiateMsg = cw721_base::InstantiateMsg;

// specialize ExecuteMsg with the CreatorInfo extention
pub type ExecuteMsg = cw721_base::ExecuteMsg<Extension>;

pub type QueryMsg = cw721_base::QueryMsg;
