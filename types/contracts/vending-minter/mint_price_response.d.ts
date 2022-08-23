import { Coin } from "./shared-types";

export interface MintPriceResponse {
current_price: Coin
public_price: Coin
whitelist_price?: (Coin | null)
[k: string]: unknown
}
