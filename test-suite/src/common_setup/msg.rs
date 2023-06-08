use anyhow::Error;
use cosmwasm_std::{Addr, Timestamp};

use sg2::msg::CollectionParams;
use sg_multi_test::StargazeApp;
use vending_factory::msg::VendingMinterInitMsgExtension;

pub struct MinterSetupParams<'a> {
    pub router: &'a mut StargazeApp,
    pub minter_admin: Addr,
    pub num_tokens: u32,
    pub collection_params: CollectionParams<String>,
    pub splits_addr: Option<String>,
    pub start_time: Option<Timestamp>,
    pub minter_code_id: u64,
    pub factory_code_id: u64,
    pub sg721_code_id: u64,
    pub init_msg: Option<VendingMinterInitMsgExtension>,
}
pub struct MinterCollectionResponse {
    pub minter: Option<Addr>,
    pub collection: Option<Addr>,
    pub factory: Option<Addr>,
    pub error: Option<Error>,
}

pub struct MinterInstantiateParams {
    pub num_tokens: u32,
    pub start_time: Option<Timestamp>,
    pub splits_addr: Option<String>,
    pub init_msg: Option<VendingMinterInitMsgExtension>,
}

use cosmwasm_schema::cw_serde;
#[cw_serde]
pub struct CodeIds {
    pub minter_code_id: u64,
    pub factory_code_id: u64,
    pub sg721_code_id: u64,
}

pub struct MinterTemplateResponse<T> {
    pub collection_response_vec: Vec<MinterCollectionResponse>,
    pub router: StargazeApp,
    pub accts: T,
}

pub struct Accounts {
    pub creator: Addr,
    pub buyer: Addr,
}
