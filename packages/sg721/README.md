# SG-721

SG-721 is a cw721-compatible spec that adds on-chain contract metadata, including royalties.

```rs
pub struct CollectionInfo<T> {
    pub creator: String,
    pub description: String,
    pub image: String,
    pub external_link: Option<String>,
    pub royalty_info: Option<T>,
}

pub struct RoyaltyInfo {
    pub payment_address: Addr,
    pub share: Decimal,
}

```

The above is set when the contract is instantiated. The contract inherits everything else from cw721-base.
