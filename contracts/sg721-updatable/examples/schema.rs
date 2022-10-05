use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use cw721_base::Extension;
use sg721::InstantiateMsg;
use sg721_base::msg::QueryMsg;
use sg721_updatable::msg::ExecuteMsg;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg<Extension>), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
}
