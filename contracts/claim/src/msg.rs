use cosmwasm_std::{to_binary, Binary, StdResult, WasmMsg};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sg_std::CosmosMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub marketplace_addr: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    #[serde(rename = "claim_mint_nft")]
    ClaimMintNFT {
        minter_address: String,
    },
    SaleFinalizedHook(SaleFinalizedHookMsg),
}

// TODO: This is duplicated from the marketplace contract until it is open sourced
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct SaleFinalizedHookMsg {
    pub collection: String,
    pub token_id: u32,
    pub seller: String,
    pub buyer: String,
}

impl SaleFinalizedHookMsg {
    pub fn new(collection: String, token_id: u32, seller: String, buyer: String) -> Self {
        SaleFinalizedHookMsg {
            collection,
            token_id,
            seller,
            buyer,
        }
    }

    /// serializes the message
    pub fn into_binary(self) -> StdResult<Binary> {
        let msg = SaleFinalizedExecuteMsg::SaleFinalizedHook(self);
        to_binary(&msg)
    }

    /// creates a cosmos_msg sending this struct to the named contract
    pub fn into_cosmos_msg<T: Into<String>>(self, contract_addr: T) -> StdResult<CosmosMsg> {
        let msg = self.into_binary()?;
        let execute = WasmMsg::Execute {
            contract_addr: contract_addr.into(),
            msg,
            funds: vec![],
        };
        Ok(execute.into())
    }
}

// This is just a helper to properly serialize the above message
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
enum SaleFinalizedExecuteMsg {
    SaleFinalizedHook(SaleFinalizedHookMsg),
}
