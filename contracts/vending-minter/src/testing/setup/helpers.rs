use cw4::Member;

pub fn member<T: Into<String>>(addr: T, weight: u64) -> Member {
    Member {
        addr: addr.into(),
        weight,
    }
}
