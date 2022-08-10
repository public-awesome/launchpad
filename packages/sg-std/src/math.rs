use cosmwasm_std::{Decimal, Uint128};

pub trait U64Ext {
    fn bps_to_decimal(self) -> Decimal;
}

impl U64Ext for u64 {
    fn bps_to_decimal(self) -> Decimal {
        Decimal::percent(self) / Uint128::from(100u128)
    }
}
