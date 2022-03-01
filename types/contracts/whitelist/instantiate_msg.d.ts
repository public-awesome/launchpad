import { Coin, Expiration } from "./shared-types";

export interface InstantiateMsg {
end_time: Expiration
members: string[]
per_address_limit: number
start_time: Expiration
unit_price: Coin
[k: string]: unknown
}
