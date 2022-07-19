# SG3 Spec: Stargaze Minter Contracts

A minimum, Stargaze minters should specify `Mint {}` that takes in 0 to many arguments.

```rs
pub enum ExecuteMsg {
    Mint {},
}
```
