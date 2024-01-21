# Stargaze Vending Minter Contract

A minter that best works for generated art collections. It's designed for collections stored on IPFS that have a base URI root.

Mints are in random order. The entire collection is shuffled on instantiation. Each mint triggers a smaller "baby" shuffle. At any time, a `Shuffle {}` function can be called to add a time element to the random mint.

## TODO

- [x] Unit tests
- [ ] Integration tests
- [ ] e2e tests


## Notes

Cannot have unit tests for this minter since the `instantiate` function checks that the contract is instantiated from a factory. Therefore we start with integration / multi tests.
