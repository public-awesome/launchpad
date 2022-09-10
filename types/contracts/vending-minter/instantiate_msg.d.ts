import { Coin, Timestamp } from "./shared-types";

/**
 * A fixed-point decimal value with 18 fractional digits, i.e. Decimal(1_000_000_000_000_000_000) == 1.0
 * 
 * The greatest possible value that can be represented is 340282366920938463463.374607431768211455 (which is (2^128 - 1) / 10^18)
 */
export type Decimal = string

export interface InstantiateMsg {
create_msg: CreateMinterMsgFor_VendingMinterInitMsgExtension
params: MinterParamsFor_ParamsExtension
[k: string]: unknown
}
export interface CreateMinterMsgFor_VendingMinterInitMsgExtension {
collection_params: CollectionParams
init_msg: VendingMinterInitMsgExtension
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
export interface VendingMinterInitMsgExtension {
base_token_uri: string
mint_price: Coin
num_tokens: number
payment_address?: (string | null)
per_address_limit: number
start_time: Timestamp
whitelist?: (string | null)
[k: string]: unknown
}

/**
 * Common params for all minters used for storage
 */
export interface MinterParamsFor_ParamsExtension {
/**
 * The minter code id
 */
code_id: number
creation_fee: Coin
extension: ParamsExtension
min_mint_price: Coin
mint_fee_bps: number
[k: string]: unknown
}
/**
 * Parameters common to all vending minters, as determined by governance
 */
export interface ParamsExtension {
airdrop_mint_fee_bps: number
airdrop_mint_price: Coin
max_per_address_limit: number
max_token_limit: number
shuffle_fee: Coin
[k: string]: unknown
}
