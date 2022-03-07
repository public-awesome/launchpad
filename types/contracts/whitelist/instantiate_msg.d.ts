import { Coin, Timestamp } from "./shared-types";

export interface InstantiateMsg {
end_time: Timestamp
member_limit: number
members: string[]
per_address_limit: number
start_time: Timestamp
unit_price: Coin
[k: string]: unknown
}
