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
params: MinterParamsFor_ParamsExtension
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
export interface Coin {
amount: Uint128
denom: string
[k: string]: unknown
}
/**
 * Parameters common to all vending minters, as determined by governance
 */
export interface ParamsExtension {
shuffle_fee: Coin
[k: string]: unknown
}
