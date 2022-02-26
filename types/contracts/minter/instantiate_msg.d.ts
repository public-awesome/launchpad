import { Addr, Coin, Config_1, Expiration } from "./shared-types";

/**
 * A fixed-point decimal value with 18 fractional digits, i.e. Decimal(1_000_000_000_000_000_000) == 1.0
 * 
 * The greatest possible value that can be represented is 340282366920938463463.374607431768211455 (which is (2^128 - 1) / 10^18)
 */
export type Decimal = string

export interface InstantiateMsg {
base_token_uri: string
batch_mint_limit?: (number | null)
num_tokens: number
per_address_limit?: (number | null)
sg721_code_id: number
sg721_instantiate_msg: InstantiateMsg1
start_time?: (Expiration | null)
unit_price: Coin
whitelist?: (string | null)
[k: string]: unknown
}
export interface InstantiateMsg1 {
config?: (Config_1 | null)
minter: string
name: string
symbol: string
[k: string]: unknown
}

export interface RoyaltyInfo {
payment_address: Addr
share: Decimal
[k: string]: unknown
}
