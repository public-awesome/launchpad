export type ExecuteMsg = ({
claim_mint_nft: {
minter_address: string
[k: string]: unknown
}
} | {
sale_finalized_hook: SaleFinalizedHookMsg
} | {
update_admin: {
admin?: (string | null)
[k: string]: unknown
}
} | {
update_marketplace: {
marketplace_addr?: (string | null)
[k: string]: unknown
}
})
/**
 * A thin wrapper around u128 that is using strings for JSON encoding/decoding, such that the full u128 range can be used for clients that convert JSON numbers to floats, like JavaScript and jq.
 * 
 * # Examples
 * 
 * Use `from` to create instances of this and `u128` to get the value out:
 * 
 * ``` # use cosmwasm_std::Uint128; let a = Uint128::from(123u128); assert_eq!(a.u128(), 123);
 * 
 * let b = Uint128::from(42u64); assert_eq!(b.u128(), 42);
 * 
 * let c = Uint128::from(70u32); assert_eq!(c.u128(), 70); ```
 */
export type Uint128 = string

export interface SaleFinalizedHookMsg {
buyer: string
collection: string
price: Coin
seller: string
token_id: number
[k: string]: unknown
}
export interface Coin {
amount: Uint128
denom: string
[k: string]: unknown
}
