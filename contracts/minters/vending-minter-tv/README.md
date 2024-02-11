# Stargaze Token Vault Vending Minter Contract

A minter that best works for generated art collections. It's designed for collections stored on IPFS that have a base URI root.

Mints are in random order. The entire collection is shuffled on instantiation. Each mint triggers a smaller "baby" shuffle. At any time, a `Shuffle {}` function can be called to add a time element to the random mint.

## TODO

- [ ] Add a `distribute` function that limits distributing more tokens than the
principal.
- [ ] Fork cw-vesting.
- [ ] Add `cw_ownable` to cw_vesting and gate `distribute`.
- [ ] Override burn in sg721-tv to allow for returning the principal.
