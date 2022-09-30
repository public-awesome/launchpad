use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, export_schema_with_title, remove_schemas, schema_for};

use base_minter::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg};
use base_minter::state::Config;
use sg4::{QueryMsg, StatusResponse};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(Config), &out_dir);
    export_schema_with_title(&schema_for!(ConfigResponse), &out_dir, "ConfigResponse");
    export_schema_with_title(&schema_for!(StatusResponse), &out_dir, "StatusResponse");
}
