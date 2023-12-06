use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Storage};

#[cw_serde]
pub struct Metadata {
    pub balance: Addr,
}
