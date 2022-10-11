use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use sg4::StatusResponse;
use vending_minter::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, MintCountResponse, MintPriceResponse,
    MintableNumTokensResponse, QueryMsg, StartTimeResponse,
};
use vending_minter::state::Config;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(Config), &out_dir);
    export_schema(&schema_for!(ConfigResponse), &out_dir);
    export_schema(&schema_for!(MintableNumTokensResponse), &out_dir);
    export_schema(&schema_for!(MintCountResponse), &out_dir);
    export_schema(&schema_for!(StartTimeResponse), &out_dir);
    export_schema(&schema_for!(MintPriceResponse), &out_dir);
    export_schema(&schema_for!(StatusResponse), &out_dir);
}
