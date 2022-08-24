import { Coin, Empty } from "./shared-types";

/**
 * A fixed-point decimal value with 18 fractional digits, i.e. Decimal(1_000_000_000_000_000_000) == 1.0
 * 
 * The greatest possible value that can be represented is 340282366920938463463.374607431768211455 (which is (2^128 - 1) / 10^18)
 */
export type Decimal = string

export interface InstantiateMsg {
create_msg: CreateMinterMsgFor_Nullable_Empty
params: MinterParamsFor_Nullable_Empty
[k: string]: unknown
}
export interface CreateMinterMsgFor_Nullable_Empty {
collection_params: CollectionParams
init_msg?: (Empty | null)
[k: string]: unknown
}
export interface CollectionParams {
/**
 * The collection code id
 */
code_id: number
info: CollectionInfoFor_RoyaltyInfoResponse
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

/**
 * Common params for all minters used for storage
 */
export interface MinterParamsFor_Nullable_Empty {
/**
 * The minter code id
 */
code_id: number
creation_fee: Coin
extension?: (Empty | null)
min_mint_price: Coin
mint_fee_bps: number
[k: string]: unknown
}
