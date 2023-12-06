# Token-vault Collection (sg721-tv)

A Token-vault collection is an NFT collection that contains a balance of fungible tokens per NFT. This balance is backed by a `cw-vesting` contract. The vesting contract is stored in the metadata for each NFT. Native tokens (i.e: STARS), can be staked to a validator while vesting to earn staking rewards.

```rs
pub struct Metadata {
    pub balance: String, // vesting contract
}
```

The vesting contract is instantiated when the NFT is minted, and funded from tokens in a minter contract. `instantiate2` is used to instantiate the vesting contract, seeded with the collection address and `token_id`. This way the minter can populate the full metadata for the NFT at mint time.

## Token-vault Minter (vending-minter-tv)

A token-vault minter is a fork of a vending minter. It must be funded with the tokens needed for mints beforehand, otherwise, minting would fail.

```rs
pub struct InstantiateMsg {
    token_balance: Coin, // amount of embedded tokens for each NFT
    // ..
}
```
