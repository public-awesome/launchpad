use cosmwasm_std::{Addr, Timestamp, Uint128};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Stake {
    pub owner: Addr,
    pub validator: Addr,
    pub end_time: Timestamp,
    pub amount: Uint128,
    /// This is the minimum amount we will pull out to reinvest + claim
    pub min_withdrawal: Uint128,
}

pub const STAKE: Item<Stake> = Item::new("stake");
