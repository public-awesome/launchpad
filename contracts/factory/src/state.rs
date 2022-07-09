use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

use vending::SudoParams;

pub const SUDO_PARAMS: Item<SudoParams> = Item::new("sudo-params");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Minter {
    pub verified: bool,
    pub blocked: bool,
}

pub type CodeID = u64;
pub type MinterAddress = Addr;

pub const MINTERS: Map<(CodeID, &MinterAddress), Minter> = Map::new("m");

// pub const COLLECTION_ADDRESS: Item<Addr> = Item::new("collection_address");

// TODO: create a map of creator addresses
