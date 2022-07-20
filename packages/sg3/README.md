# SG3 Spec: Stargaze Minter Contracts

A minter must store the factory that created it, so it can query its parameters:

```rs
pub struct MinterConfig<T> {
    pub factory: Addr,
    pub collection_code_id: u64,
    pub extension: T,
}
```

Custom minters can add more fields using `extention`.

A minimum, Stargaze minters should specify at least one `Mint {}` operation that takes 0 to many arguments.

```rs
pub enum ExecuteMsg {
    Mint {},
}
```
