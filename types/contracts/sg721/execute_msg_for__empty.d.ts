import { Empty, Expiration } from "./shared-types";

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
