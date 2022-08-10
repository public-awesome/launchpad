import { Coin, Timestamp } from "./shared-types";

/**
 * A fixed-point decimal value with 18 fractional digits, i.e. Decimal(1_000_000_000_000_000_000) == 1.0
 * 
 * The greatest possible value that can be represented is 340282366920938463463.374607431768211455 (which is (2^128 - 1) / 10^18)
 */
export type Decimal = string

export interface InstantiateMsg {
create_msg: CreateMinterMsgFor_MinterInitMsgExtension
params: MinterParamsFor_ParamsExtension
[k: string]: unknown
}
export interface CreateMinterMsgFor_MinterInitMsgExtension {
collection_params: CollectionParams
init_msg: MinterInitMsgExtension
[k: string]: unknown
}
export interface CollectionParams {
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
 * Properties of each specific vending minter
 */
export interface MinterInitMsgExtension {
base_token_uri: string
num_tokens: number
per_address_limit: number
start_time: Timestamp
unit_price: Coin
whitelist?: (string | null)
[k: string]: unknown
}

/**
 * Common params for all minters, updatable by governance
 */
export interface MinterParamsFor_ParamsExtension {
airdrop_mint_fee_bps: number
airdrop_mint_price: Coin
code_id: number
creation_fee: Coin
extension: ParamsExtension
max_per_address_limit: number
max_token_limit: number
min_mint_price: Coin
mint_fee_bps: number
[k: string]: unknown
}
/**
 * Parameters common to all vending minters, as determined by governance
 */
export interface ParamsExtension {
shuffle_fee: Coin
[k: string]: unknown
}
