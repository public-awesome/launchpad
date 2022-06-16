use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use cw20_ics20::msg::{ListChannelsResponse, PortResponse};

use ics721::msg::{ChannelResponse, ExecuteMsg, InstantiateMsg, QueryMsg, TokensResponse};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(ChannelResponse), &out_dir);
    export_schema(&schema_for!(TokensResponse), &out_dir);
    export_schema(&schema_for!(ListChannelsResponse), &out_dir);
    export_schema(&schema_for!(PortResponse), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
}
