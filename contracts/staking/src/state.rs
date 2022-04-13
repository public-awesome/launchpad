use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

/// address -> balance
pub const BALANCE: Map<&Addr, Uint128> = Map::new("b");

/// (address, validator) -> stake amount
pub const STAKE: Map<(&Addr, &Addr), Uint128> = Map::new("s");
