# Base Minter Factory

A simple contract that maintains all base 1/1 minter governance parameters.

It's responsible for creating new minters with the latest governance parameters.

It also maintains verified, blocked, and explicit lists for vending minters.

```rs
pub struct Minter {
    pub verified: bool,
    pub blocked: bool,
    pub is_explicit: bool,
}
```

Minters can be verified, blocked, or deemed explicit via on-chain governance.
