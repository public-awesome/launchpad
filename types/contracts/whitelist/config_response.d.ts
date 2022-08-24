import { Coin, Timestamp } from "./shared-types";

export interface ConfigResponse {
end_time: Timestamp
is_active: boolean
member_limit: number
mint_price: Coin
num_members: number
per_address_limit: number
start_time: Timestamp
[k: string]: unknown
}
