use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, WasmMsg};

#[cw_serde]
pub enum Admin {
    Address { addr: String },
    Creator {},
}

#[cw_serde]
pub struct ContractInstantiateMsg {
    pub code_id: u64,
    pub msg: Binary,
    pub admin: Option<Admin>,
    pub label: String,
}

impl ContractInstantiateMsg {
    pub fn into_wasm_msg(self, creator: Addr) -> WasmMsg {
        WasmMsg::Instantiate {
            admin: self.admin.map(|admin| match admin {
                Admin::Address { addr } => addr,
                Admin::Creator {} => creator.into_string(),
            }),
            code_id: self.code_id,
            msg: self.msg,
            label: self.label,
            funds: vec![],
        }
    }
}
