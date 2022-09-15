# SG4 Spec: Stargaze Minter Contracts

A minter must store the factory that created it, so it can query its parameters:

```rs
pub struct MinterConfig<T> {
    pub factory: Addr,
    pub collection_code_id: u64,
    pub mint_price: Coin,
    pub extension: T,
}
```

Custom minters can add more fields using `extension`.

A minimum, Stargaze minters should specify at least one `Mint {}` operation that takes 0 to many arguments.

```rs
pub enum ExecuteMsg {
    Mint {},
}
```

Provides minter status for each collection. Status is changed through on-chain governance proposals.

- Verified: a community based signal that the creators are verified
- Blocked: a community based signal that the collection should be blocked
- Explicit: a community based signal that the collection has explicit content (not safe for work, pornographic, etc)

```rs
pub struct Minter {
    pub verified: bool,
    pub blocked: bool,
    pub is_explicit: bool,
}
```
