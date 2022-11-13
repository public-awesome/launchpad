use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use sg_eth_airdrop::msg::{
    AirdropClaimResponse, ExecuteMsg, InstantiateMsg, QueryMsg, VerifyResponse,
};
use vending_minter::msg::{
    ConfigResponse as VendingConfigResponse, ExecuteMsg as VendingMinterExecuteMessage,
};
use vending_minter::state::Config as VendingConfig;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(AirdropClaimResponse), &out_dir);
    export_schema(&schema_for!(VerifyResponse), &out_dir);
    export_schema(&schema_for!(VendingConfig), &out_dir);
    export_schema(&schema_for!(VendingConfigResponse), &out_dir);
    export_schema(&schema_for!(VendingMinterExecuteMessage), &out_dir);
}
