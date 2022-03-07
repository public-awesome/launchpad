import { Coin, Timestamp } from "./shared-types";

export interface ConfigResponse {
end_time: Timestamp
is_active: boolean
member_limit: number
num_members: number
per_address_limit: number
start_time: Timestamp
unit_price: Coin
[k: string]: unknown
}
