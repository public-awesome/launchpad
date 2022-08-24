import { Coin, Timestamp } from "./shared-types";

export interface InstantiateMsg {
end_time: Timestamp
member_limit: number
members: string[]
mint_price: Coin
per_address_limit: number
start_time: Timestamp
[k: string]: unknown
}
