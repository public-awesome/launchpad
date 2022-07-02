use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal};
use cw_storage_plus::{Item, Map};
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub count: i32,
    pub owner: Addr,
}

pub const STATE: Item<State> = Item::new("state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VendingMinterParams {
    pub max_token_limit: u32,
    pub max_per_address_limit: u32,
    pub min_mint_price: u128,
    pub airdrop_mint_price: u128,
    pub mint_fee_percent: Decimal,
    pub airdrop_mint_fee_percent: Decimal,
    pub shuffle_fee: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SudoParams {
    /// A list of allowed minter code IDs
    pub minter_codes: Vec<u64>,
    pub vending_minter: VendingMinterParams,
}

pub const SUDO_PARAMS: Item<SudoParams> = Item::new("sudo-params");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Minter {
    pub verified: bool,
    pub blocked: bool,
}

pub type CodeID = u64;
pub type MinterAddress = Addr;

pub const MINTERS: Map<(CodeID, &MinterAddress), Minter> = Map::new("m");
