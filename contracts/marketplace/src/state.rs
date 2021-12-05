use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::Map;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Bid {
    // Amount of the currency being bid
    pub amount: Coin,
    // Address to the cw20 token being used to bid
    pub bidder: Addr,
    // Address of the recipient
    pub recipient: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Ask {
    // Amount of the currency being asked
    pub amount: Coin,
}

// Mapping from (media_contract, token_id, bidder) to bid
pub const TOKEN_BIDS: Map<(&Addr, &str, &Addr), Bid> = Map::new("token_bidders");

// Mapping from  (media_contract, token_id) to the current ask for the token
pub const TOKEN_ASKS: Map<(&Addr, &str), Ask> = Map::new("token_asks");
