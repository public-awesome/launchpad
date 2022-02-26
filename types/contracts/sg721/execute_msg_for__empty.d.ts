/**
 * This is like Cw721ExecuteMsg but we add a Mint command for an owner to make this stand-alone. You will likely want to remove mint and use other control logic in any contract that inherits this.
 */
export type ExecuteMsgFor_Empty = ({
transfer_nft: {
recipient: string
token_id: string
[k: string]: unknown
}
} | {
send_nft: {
contract: string
msg: Binary
token_id: string
[k: string]: unknown
}
} | {
approve: {
expires?: (Expiration | null)
spender: string
token_id: string
[k: string]: unknown
}
} | {
revoke: {
spender: string
token_id: string
[k: string]: unknown
}
} | {
approve_all: {
expires?: (Expiration | null)
operator: string
[k: string]: unknown
}
} | {
revoke_all: {
operator: string
[k: string]: unknown
}
} | {
mint: MintMsgFor_Empty
} | {
burn: {
token_id: string
[k: string]: unknown
}
})
/**
 * Binary is a wrapper around Vec<u8> to add base64 de/serialization with serde. It also adds some helper methods to help encode inline.
 * 
 * This is only needed as serde-json-{core,wasm} has a horrible encoding for Vec<u8>
 */
export type Binary = string
/**
 * Expiration represents a point in time when some event happens. It can compare with a BlockInfo and will return is_expired() == true once the condition is hit (and for every block in the future)
 */
export type Expiration = ({
at_height: number
} | {
at_time: Timestamp
} | {
never: {
[k: string]: unknown
}
})
/**
 * A point in time in nanosecond precision.
 * 
 * This type can represent times from 1970-01-01T00:00:00Z to 2554-07-21T23:34:33Z.
 * 
 * ## Examples
 * 
 * ``` # use cosmwasm_std::Timestamp; let ts = Timestamp::from_nanos(1_000_000_202); assert_eq!(ts.nanos(), 1_000_000_202); assert_eq!(ts.seconds(), 1); assert_eq!(ts.subsec_nanos(), 202);
 * 
 * let ts = ts.plus_seconds(2); assert_eq!(ts.nanos(), 3_000_000_202); assert_eq!(ts.seconds(), 3); assert_eq!(ts.subsec_nanos(), 202); ```
 */
export type Timestamp = Uint64
/**
 * A thin wrapper around u64 that is using strings for JSON encoding/decoding, such that the full u64 range can be used for clients that convert JSON numbers to floats, like JavaScript and jq.
 * 
 * # Examples
 * 
 * Use `from` to create instances of this and `u64` to get the value out:
 * 
 * ``` # use cosmwasm_std::Uint64; let a = Uint64::from(42u64); assert_eq!(a.u64(), 42);
 * 
 * let b = Uint64::from(70u32); assert_eq!(b.u64(), 70); ```
 */
export type Uint64 = string

export interface MintMsgFor_Empty {
/**
 * Any custom extension used by this contract
 */
extension: Empty
/**
 * The owner of the newly minter NFT
 */
owner: string
/**
 * Unique ID of the NFT
 */
token_id: string
/**
 * Universal resource identifier for this NFT Should point to a JSON file that conforms to the ERC721 Metadata JSON Schema
 */
token_uri?: (string | null)
[k: string]: unknown
}
/**
 * An empty struct that serves as a placeholder in different places, such as contracts that don't set a custom message.
 * 
 * It is designed to be expressable in correct JSON and JSON Schema but contains no meaningful data. Previously we used enums without cases, but those cannot represented as valid JSON Schema (https://github.com/CosmWasm/cosmwasm/issues/451)
 */
export interface Empty {
[k: string]: unknown
}
