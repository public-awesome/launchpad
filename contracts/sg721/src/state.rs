use crate::ContractError;
use cosmwasm_std::{Addr, Decimal};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub contract_uri: String,
    pub creator: Addr,
    pub royalties: Option<RoyaltyInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RoyaltyInfo {
    pub payment_address: Addr,
    pub share: Decimal,
}

impl RoyaltyInfo {
    // Checks if RoyaltyInfo is valid
    pub fn is_valid(&self) -> Result<bool, ContractError> {
        if self.share > Decimal::one() {
            return Err(ContractError::InvalidRoyalities {});
        }

        if self.share < Decimal::zero() {
            return Err(ContractError::InvalidRoyalities {});
        }

        Ok(true)
    }
}

pub const COLLECTION_INFO: Item<Config> = Item::new("collection_info");
