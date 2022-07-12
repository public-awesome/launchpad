# Vending Minter Factory

A contract that maintains all vending machine minter governance parameters.

It's responsible for creating new minters with the latest governance parameters.

It also maintains verified and blocked lists for vending minters.

```rs
pub struct Minter {
    pub verified: bool,
    pub blocked: bool,
}
```

Minters can be verified or blocked via on-chain governance.
