use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, export_schema_with_title, remove_schemas, schema_for};

use cw721::{
    CollectionMetadataAndExtension, CollectionMetadataExtensionWrapper,
    DefaultOptionCollectionMetadataExtension, DefaultOptionNftMetadataExtension, RoyaltyInfo,
};
#[allow(deprecated)]
pub use cw721_base::{
    msg::{
        AllNftInfoResponse, ApprovalResponse, ApprovalsResponse, MinterResponse, NftInfoResponse,
        NumTokensResponse, OperatorsResponse, OwnerOfResponse, TokensResponse,
    },
    ContractInfoResponse,
};
use sg721::InstantiateMsg;
#[allow(deprecated)]
pub use sg721_base::msg::CollectionInfoResponse;
use sg721_base::msg::QueryMsg;
use sg721_updatable::msg::ExecuteMsg;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(
        &schema_for!(ExecuteMsg<DefaultOptionNftMetadataExtension, DefaultOptionCollectionMetadataExtension>),
        &out_dir,
    );
    export_schema(
        &schema_for!(
            QueryMsg::<DefaultOptionNftMetadataExtension, DefaultOptionCollectionMetadataExtension>
        ),
        &out_dir,
    );
    #[allow(deprecated)]
    export_schema(&schema_for!(CollectionInfoResponse), &out_dir);
    export_schema(
        &schema_for!(
            CollectionMetadataAndExtension<CollectionMetadataExtensionWrapper<RoyaltyInfo>>
        ),
        &out_dir,
    );
    export_schema_with_title(
        &schema_for!(AllNftInfoResponse<DefaultOptionNftMetadataExtension>),
        &out_dir,
        "AllNftInfoResponse",
    );
    export_schema_with_title(&schema_for!(TokensResponse), &out_dir, "AllTokensResponse");
    export_schema_with_title(
        &schema_for!(OperatorsResponse),
        &out_dir,
        "AllOperatorsResponse",
    );
    export_schema(&schema_for!(MinterResponse), &out_dir);
    export_schema(&schema_for!(ApprovalResponse), &out_dir);
    export_schema(&schema_for!(ApprovalsResponse), &out_dir);
    #[allow(deprecated)]
    export_schema(&schema_for!(ContractInfoResponse), &out_dir);
    export_schema(
        &schema_for!(CollectionMetadataAndExtension<DefaultOptionCollectionMetadataExtension>),
        &out_dir,
    );
    export_schema_with_title(
        &schema_for!(NftInfoResponse<DefaultOptionNftMetadataExtension>),
        &out_dir,
        "NftInfoResponse",
    );
    export_schema(&schema_for!(NumTokensResponse), &out_dir);
    export_schema(&schema_for!(OwnerOfResponse), &out_dir);
    export_schema(&schema_for!(TokensResponse), &out_dir);
}
