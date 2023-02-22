use cosmwasm_std::{Addr, Coin};
use cw4::Member;
use cw_multi_test::{App, AppBuilder};

const OWNER: &str = "admin0001";

pub fn mock_app_builder_init_funds(init_funds: &[Coin]) -> App {
    AppBuilder::new().build(|router, _, storage| {
        router
            .bank
            .init_balance(storage, &Addr::unchecked(OWNER), init_funds.to_vec())
            .unwrap();
    })
}

pub fn member<T: Into<String>>(addr: T, weight: u64) -> Member {
    Member {
        addr: addr.into(),
        weight,
    }
}
