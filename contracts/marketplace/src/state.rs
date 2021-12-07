use crate::error::ContractError;
use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::Map;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Bid {
    // Amount of the currency being bid
    pub amount: Coin,
    // Address to the cw20 token being used to bid
    pub bidder: Addr,
    // Address of the recipient
    pub recipient: Addr,
}

impl Bid {
    /// Checks if a bid is valid
    pub fn is_valid(&self) -> Result<bool, ContractError> {
        let bid_amount = &self.amount;

        // Check amount is not zero
        if bid_amount.amount.is_zero() {
            return Err(ContractError::InvalidBidTooLow {});
        }

        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Ask {
    // Amount of the currency being asked
    pub amount: Coin,
}

// Mapping from (media_contract, token_id, bidder) to bid
pub const TOKEN_BIDS: Map<(&Addr, &str, &Addr), Bid> = Map::new("token_bidders");

// Mapping from  (media_contract, token_id) to the current ask for the token
pub const TOKEN_ASKS: Map<(&Addr, &str), Ask> = Map::new("token_asks");

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{coin, Addr};
    const NATIVE_TOKEN_DENOM: &str = "ustars";

    #[test]
    fn valid_bids() {
        let bidder: Addr = Addr::unchecked("bidder");
        let creator: Addr = Addr::unchecked("creator");

        // Normal bid
        let bid = Bid {
            amount: coin(100, NATIVE_TOKEN_DENOM),
            bidder: bidder.clone(),
            recipient: creator.clone(),
        };
        assert!(bid.is_valid().is_ok());

        // High number
        let bid = Bid {
            amount: coin(1000000000000, NATIVE_TOKEN_DENOM),
            bidder,
            recipient: creator,
        };
        assert!(bid.is_valid().is_ok());
    }

    #[test]
    fn invalid_bids() {
        let bidder: Addr = Addr::unchecked("bidder");
        let creator: Addr = Addr::unchecked("creator");

        // Low number
        let bid = Bid {
            amount: coin(0, NATIVE_TOKEN_DENOM),
            bidder,
            recipient: creator,
        };
        assert!(bid.is_valid().is_err());
    }
}
