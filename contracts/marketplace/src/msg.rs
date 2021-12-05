use crate::state::{Ask, Bid};
use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns the current asking price for a token
    CurrentAsk { collection: Addr, token_id: String },
    /// Returns the bid for a token / bidder
    Bid {
        collection: Addr,
        token_id: String,
        bidder: Addr,
    },
    /// Returns list of bids for token
    Bids {
        collection: Addr,
        token_id: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    SetBid {
        collection: Addr,
        token_id: String,
        bid: Bid,
    },
    RemoveBid {
        collection: Addr,
        token_id: String,
        bidder: Addr,
    },
    SetAsk {
        collection: Addr,
        token_id: String,
        ask: Ask,
    },
    RemoveAsk {
        collection: Addr,
        token_id: String,
    },
    AcceptBid {
        collection: Addr,
        token_id: String,
        bid: Bid,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CurrentAskResponse {
    pub ask: Option<Ask>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BidResponse {
    pub bid: Option<Bid>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BidsResponse {
    pub bids: Vec<Bid>,
}
