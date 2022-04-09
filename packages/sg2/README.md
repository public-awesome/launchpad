# SG2 Spec: Contract Parameter Governance

This spec defines an interface for chain-governed contract parameters. It enables Cosmos chain governance to vote on contract parameter updates.

## Messages

A series of messages to update values of various types. These messages are only exposed to chain govnernance and cannot be called by users or other smart contracts.

`UpdateParamu32{contract_name, key, value}`

- `contract_name` is the name of the contract defined in cw2.

- `key` is a string that uniquely identifies the parameter

- `value` is an integer value for the parameter

## Queries

`GetParamu32{contract_name, key}` - Returns the value for the contract name and key.
