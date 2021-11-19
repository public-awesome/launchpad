use cosmwasm_std::{Addr, Coin, Empty};
use cw721_base::Cw721Contract;
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

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub enum Order {
    Bid(Bid),
    Ask(Ask),
}
pub type Extension = Order;
pub type Cw721BaseContract<'a> = Cw721Contract<'a, Extension, Empty>;

// [TODO]
// need to be able to iterate over (collection, token_id)
// two separate contracts for bids and asks?
