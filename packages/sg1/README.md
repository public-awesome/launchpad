# SG1 Spec: Fair Burn

Fair Burn is a specification for processing fees in Stargaze, influenced by [EIP-1559](https://eips.ethereum.org/EIPS/eip-1559).

With Fair Burn, a portion of fees are burned, and the remaining portion is distributed to stakers. Currently, 50% is burned, and 50% go to the Community Pool.

## Governance Parameters

```rs
const FEE_BURN_PERCENT: u64 = 50;
```

## API

Contracts can use Fair Burn via one of the following functions:

```rs
/// Burn and distribute fees and return an error if the fee is not enough
checked_fair_burn(info: &MessageInfo, fee_amount: u128) -> Result<Vec<SubMsg>, FeeError>

/// Burn and distribute fees, assuming the right fee is passed in
fair_burn(fee_amount: u128) -> Vec<SubMsg>
```
