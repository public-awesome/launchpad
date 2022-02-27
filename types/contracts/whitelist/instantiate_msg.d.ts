import { Coin, Expiration } from "./shared-types";

export interface InstantiateMsg {
end_time: Expiration
members: string[]
start_time: Expiration
unit_price: Coin
[k: string]: unknown
}
