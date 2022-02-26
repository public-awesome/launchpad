import { Addr, Coin, Expiration } from "./shared-types";

export interface ConfigResponse {
admin: Addr
base_token_uri: string
batch_mint_limit?: (number | null)
num_tokens: number
per_address_limit?: (number | null)
sg721_address: Addr
sg721_code_id: number
start_time?: (Expiration | null)
unit_price: Coin
whitelist?: (Addr | null)
[k: string]: unknown
}
