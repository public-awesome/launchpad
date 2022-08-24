import { Addr, Coin, Timestamp } from "./shared-types";

/**
 * Saved in every minter
 */
export interface MinterConfigFor_ConfigExtension {
collection_code_id: number
extension: ConfigExtension
factory: Addr
mint_price: Coin
[k: string]: unknown
}
export interface ConfigExtension {
admin: Addr
base_token_uri: string
num_tokens: number
per_address_limit: number
start_time: Timestamp
whitelist?: (Addr | null)
[k: string]: unknown
}
