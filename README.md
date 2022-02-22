# Stargaze CosmWasm Contracts

Stargaze smart contracts are written in [CosmWasm](https://cosmwasm.com), a multi-chain smart contracting platform in Rust.

Contracts run in a WASM VM on the [Stargaze Layer 1 blockchain](https://github.com/public-awesome/stargaze).

## Sg-721

Stargaze's NFT contract sg721 is a set of optional extensions on top of [cw721-base](https://github.com/CosmWasm/cw-nfts/tree/main/contracts/cw721-base), and conforms to the [cw721 specification](https://github.com/CosmWasm/cw-nfts/tree/main/packages/cw721).

## Minter

A contract that facilitates primary market vending machine style minting.

Features:

- Aidrops

- Whitelist

## Whitelist

A basic whitelist contract with a max number of members and an end time.

## Sg-Std

Stargaze standard library.
