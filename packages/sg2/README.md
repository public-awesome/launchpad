# SG2 Spec: Minter Factory Contracts

Common types needed for minter factories. New minter factories should extended these types to add their custom properties.

For example, the vending minter factory needs a `shuffle_fee`. This is added as a custom extension to `MinterParams` and `UpdateParamsMsg`.

```rs
pub struct VendingUpdateParamsExtension {
    pub shuffle_fee: Option<Coin>,
}
pub type VendingUpdateParamsMsg = UpdateParamsMsg<VendingUpdateParamsExtension>;

pub struct ParamsExtension {
    pub shuffle_fee: Coin,
}
pub type VendingMinterParams = MinterParams<ParamsExtension>;
```
