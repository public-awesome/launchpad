import { Uint128 } from "./shared-types";

/**
 * Delegate funds to a Stargaze validator. `min_duration` is in days.
 */
export interface InstantiateMsg {
/**
 * During of timelock
 */
min_duration: number
/**
 * This is the minimum amount we will pull out to reinvest + claim
 */
min_withdrawal: Uint128
/**
 * Address of validator to stake to
 */
validator: string
[k: string]: unknown
}
