use cosmwasm_std::{Addr, Timestamp, Uint128};
use cw_storage_plus::Map;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Stake {
    pub end_time: Timestamp,
    pub amount: Uint128,
}

/// (address, validator) -> stake amount
pub const STAKE: Map<(&Addr, &Addr), Stake> = Map::new("s");
