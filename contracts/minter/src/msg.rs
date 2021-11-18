use cosmwasm_std::Addr;
use cw4::Member;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub count: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Increment {},
    Reset {
        count: i32,
    },
    // Create a new NFT collection
    InitCollection {
        code_id: u64,
        name: String,
        symbol: String,
    },
    // Mint into an existing NFT collection
    Mint {
        collection: Addr,
        token_id: String,
        owner: String,
        // Storing metadata off-chain on IPFS is better for interoperability
        token_uri: Option<String>,
        // Members of a cw4-group for creator royalties
        creators: Vec<Member>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    GetCount {},
    CollectionsForCreator { creator: Addr },
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CountResponse {
    pub count: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollectionsForCreatorResponse {
    pub collections: Vec<Addr>,
}
