use cosmwasm_std::{Addr, Coin};
use cw4::Member;
use cw_multi_test::{App, AppBuilder, IntoAddr};

pub fn owner_addr() -> Addr {
    "admin0001".into_addr()
}

pub fn mock_app_builder_init_funds(init_funds: &[Coin]) -> App {
    AppBuilder::new().build(|router, _, storage| {
        router
            .bank
            .init_balance(storage, &owner_addr(), init_funds.to_vec())
            .unwrap();
    })
}

pub fn member<T: Into<String>>(addr: T, weight: u64) -> Member {
    Member {
        addr: addr.into(),
        weight,
    }
}
