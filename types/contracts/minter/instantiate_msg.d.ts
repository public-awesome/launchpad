import { Coin, Timestamp } from "./shared-types";

/**
 * A fixed-point decimal value with 18 fractional digits, i.e. Decimal(1_000_000_000_000_000_000) == 1.0
 * 
 * The greatest possible value that can be represented is 340282366920938463463.374607431768211455 (which is (2^128 - 1) / 10^18)
 */
export type Decimal = string

export interface InstantiateMsg {
base_token_uri: string
num_tokens: number
per_address_limit: number
sg721_code_id: number
sg721_instantiate_msg: InstantiateMsg1
start_time: Timestamp
unit_price: Coin
whitelist?: (string | null)
[k: string]: unknown
}
export interface InstantiateMsg1 {
collection_info: CollectionInfoFor_RoyaltyInfoResponse
minter: string
name: string
symbol: string
[k: string]: unknown
}
export interface CollectionInfoFor_RoyaltyInfoResponse {
creator: string
description: string
external_link?: (string | null)
image: string
royalty_info?: (RoyaltyInfoResponse | null)
[k: string]: unknown
}
export interface RoyaltyInfoResponse {
payment_address: string
share: Decimal
[k: string]: unknown
}
