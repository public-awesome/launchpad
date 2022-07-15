use cosmwasm_std::{Decimal, Uint128};

pub trait U64Ext {
    fn to_percent(self) -> Decimal;
}

impl U64Ext for u64 {
    fn to_percent(self) -> Decimal {
        Decimal::percent(self) / Uint128::from(100u128)
    }
}
