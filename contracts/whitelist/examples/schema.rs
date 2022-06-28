use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use sg_whitelist::msg::{
    ConfigResponse, ExecuteMsg, HasEndedResponse, HasMemberResponse, HasStartedResponse,
    InstantiateMsg, IsActiveResponse, MembersResponse, QueryMsg,
};
use sg_whitelist::state::Config;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(Config), &out_dir);
    export_schema(&schema_for!(ConfigResponse), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(HasEndedResponse), &out_dir);
    export_schema(&schema_for!(HasMemberResponse), &out_dir);
    export_schema(&schema_for!(HasStartedResponse), &out_dir);
    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(IsActiveResponse), &out_dir);
    export_schema(&schema_for!(MembersResponse), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
}
