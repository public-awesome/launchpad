use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use vending::VendingMinterParams;

pub const SUDO_PARAMS: Item<VendingMinterParams> = Item::new("sudo-params");

// pub const COLLECTION_ADDRESS: Item<Addr> = Item::new("collection_address");

// TODO: create a map of creator addresses
