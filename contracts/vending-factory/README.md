# Vending Minter Factory

A contract that maintains all vending machine minter governance parameters.

It's responsible for creating new minters with the latest governance parameters.

It also maintains verified, blocked, and explicit lists for vending minters.

```rs
pub struct Minter {
    pub verified: bool,
    pub blocked: bool,
    pub is_explicit: bool,
}
```

Minters can be verified, blocked, or deemed via on-chain governance.
