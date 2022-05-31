export type ExecuteMsg = ({
receive: Cw721ReceiveMsg
} | {
transfer: TransferMsg
})
/**
 * Binary is a wrapper around Vec<u8> to add base64 de/serialization with serde. It also adds some helper methods to help encode inline.
 * 
 * This is only needed as serde-json-{core,wasm} has a horrible encoding for Vec<u8>
 */
export type Binary = string

/**
 * Cw721ReceiveMsg should be de/serialized under `Receive()` variant in a ExecuteMsg
 */
export interface Cw721ReceiveMsg {
msg: Binary
sender: string
token_id: string
[k: string]: unknown
}
export interface TransferMsg {
channel: string
class_id: string
class_uri?: (string | null)
remote_address: string
timeout?: (number | null)
token_ids: string[]
token_uris: string[]
[k: string]: unknown
}
