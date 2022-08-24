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

export interface InstantiateMsg {
params: MinterParamsFor_Nullable_Empty
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
export interface Coin {
amount: Uint128
denom: string
[k: string]: unknown
}
/**
 * An empty struct that serves as a placeholder in different places, such as contracts that don't set a custom message.
 * 
 * It is designed to be expressable in correct JSON and JSON Schema but contains no meaningful data. Previously we used enums without cases, but those cannot represented as valid JSON Schema (https://github.com/CosmWasm/cosmwasm/issues/451)
 */
export interface Empty {
[k: string]: unknown
}
