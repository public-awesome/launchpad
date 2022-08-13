# Base Minter Factory

A simple contract that maintains all base 1/1 minter governance parameters.

It's responsible for creating new minters with the latest governance parameters.

It also maintains verified and blocked lists for vending minters.

```rs
pub struct Minter {
    pub verified: bool,
    pub blocked: bool,
}
```

Minters can be verified or blocked via on-chain governance.
