# SG721 ICS721

This is an *IBC Enabled* contract that allows sending SG721 NFTs from one CosmWasm chain over the ICS721 (draft) standard to another CosmWasm chain.

It can be extended to also send NFTs from a CosmWasm chain to the native NFT module on a Cosmos SDK chain that doesn't have CosmWasm support.

## Workflow

Similar to cw20-ics20, the contract starts with minimal state, just the default timeout for all the packets it sends.

An external party has to make one or more channels using this contract as one endpoint.

You can send any SG721 token to this contract via the [receiver pattern](https://github.com/CosmWasm/cw-nfts/blob/main/packages/cw721/src/receiver.rs).

## Messages

This contract only accepts a `Sg721ReceiveMsg` from a sg721 contract. 

```rust
pub struct Sg721ReceiveMsg {
    pub sender: String,
    pub token_id: String,
    pub msg: Binary,
}
```

The data inside the message must be JSON-serialized.

```rust
pub struct TransferMsg {
    /// The local channel to send the packets on
    pub channel: String,
    /// The remote address to send to
    /// Don't use HumanAddress as this will likely have a different Bech32 prefix than we use
    /// and cannot be validated locally
    pub remote_address: String,
    /// How long the packet lives in seconds. If not specified, use default_timeout
    pub timeout: Option<u64>,
}
```

## Queries

Queries only make sense relative to the established channels of this contract.

* `Port{}` - returns the port ID this contract has bound, so you can create channels. This info can be queried 
  via wasmd contract info query, but we expose another query here for convenience.
* `ListChannels{}` - returns a (currently unpaginated) list of all channels that have been created on this contract.
  Returns their local channelId along with some basic metadata, like the remote port/channel and the connection they
  run on top of.
* `Channel{id}` - returns more detailed information on one specific channel. In addition to the information available
  in the list view, it returns the current outstanding balance on that channel, as well as the total amount that
  has ever been sent on the channel.

## Credits

This README was adapted from the cw20-ics20 [README](https://github.com/CosmWasm/cw-plus/tree/main/contracts/cw20-ics20).
