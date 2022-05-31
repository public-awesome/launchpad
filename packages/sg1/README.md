# SG1 Spec: Fair Burn

Fair Burn is a specification for processing fees in Stargaze, influenced by [EIP-1559](https://eips.ethereum.org/EIPS/eip-1559).

With Fair Burn, a portion of fees are burned, and the remaining portion is distributed to stakers. Currently, 50% is burned, and 50% go to the Community Pool.

NOTE: In a future version, the Community Pool allocation will be distributed to stakers instead.

Fair Burn also includes a way to incentivize custom smart contract development by distributing some of the fee to a developer address. This developer fee is substracted from the amount burned.

For example, if a developer address is provided, 40% fees will be burned, 10% will go to the developer address, and 50% will go to the Community Pool.

## Governance Parameters

```rs
const FEE_BURN_PERCENT: u64 = 50;      // 50%
const DEV_INCENTIVE_PERCENT: u64 = 10; // 10%
```

## API

Contracts can use Fair Burn via one of the following functions.

```rs
/// Burn and distribute fees and return an error if the fee is not enough
checked_fair_burn(info: &MessageInfo, fee: u128, developer: Option<Addr>) -> Result<Vec<SubMsg>, FeeError>

/// Burn and distribute fees, assuming the right fee is passed in
fair_burn(fee: u128, developer: Option<Addr>) -> Vec<SubMsg>
```

Custom contract developers can pass in a a `developer` address that will receive 10% of all fees.
