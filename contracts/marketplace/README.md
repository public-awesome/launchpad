# NFT Market

The Stargaze NFT auction happens as a perpetual option. Currently, there are no enforced time limits.

Anyone can call the `SetBid` method and make an offer on _any_ NFT, the funds are sent to the marketplace contract which serves as an escrow.

Bidders can remove their bids and reclaim their funds with `RemoveBid`. When a bid is removed, it's deposit is refunded. New bids automatically remove and refund previous bids.

The NFT owner can at any point pick a bid they like and call the `AcceptBid` method, which will transfer both the funds and the NFT.

Optionally, the NFT owner can set an ask price with the `SetAsk` method. If an ask is set, any bid that meets the ask requirements is automatically accepted and transferred to the bidder. The owner can remove the ask with `RemoveAsk`.

When a bid is accepted, both the payment and NFT are automatically transferred. Payment is split up and distributed according to the Royalties specified when the NFT was minted.

### Authorizing the marketplace contract

In order to accept a bid or set an asking price, the owner needs to grant approval to the marketplace contract for transferring the NFT. This can be done with an NFT's `Approve` method for each NFT, or by using `ApproveAll` for all NFTs in the collection.

## Running this Contract

You will need Rust 1.44.1+ with `wasm32-unknown-unknown` target installed.

You can run unit tests on this via:

`cargo test`

Once you are happy with the content, you can compile it to wasm via:

```
RUSTFLAGS='-C link-arg=-s' cargo wasm
cp ../../target/wasm32-unknown-unknown/release/sg_marketplace.wasm .
ls -l sg_marketplace.wasm
sha256sum sg_marketplace.wasm
```

Or for a production-ready (optimized) build, run a build command in the repository root: https://github.com/CosmWasm/cosmwasm-plus#compiling.
