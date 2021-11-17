use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub count: i32,
    pub owner: Addr,
}

pub const STATE: Item<State> = Item::new("state");

// Mapping from (nft_contract, token_id, bidder) to bid
pub const BIDDERS: Map<(&Addr, &str, &Addr), Bid> = Map::new("token_bidders");

// Mapping from (media_contract, token_id) to the bid shares for the token
// pub const BID_SHARES: Map<(&Addr, &str), BidShares> = Map::new("bid_shares");

// Mapping from  (nft_contract, token_id) to the current ask for the token
pub const ASKS: Map<(&Addr, &str), Ask> = Map::new("token_asks");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Bid {
    // Amount of the currency being bid
    pub amount: Coin,
    // Address to the cw20 token being used to bid
    pub bidder: Addr,
    // Address of the recipient
    pub recipient: Addr,
    // % of the next sale to award the current owner
    // pub sell_on_share: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Ask {
    // Amount of the currency being asked
    pub amount: Coin,
}
