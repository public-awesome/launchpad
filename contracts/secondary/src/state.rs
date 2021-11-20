use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub count: i32,
    pub owner: Addr,
}

pub const STATE: Item<State> = Item::new("state");

// Mapping from (nft_contract, token_id, bidder) to bid
pub const BIDDERS: Map<(&Addr, &str, &Addr), Bid> = Map::new("bidders");

// Mapping from  (nft_contract, token_id) to the current ask
pub const ASKS: Map<(&Addr, &str), Ask> = Map::new("asks");

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
