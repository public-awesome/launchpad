import { Coin, Expiration } from "./shared-types";

export interface ConfigResponse {
end_time: Expiration
is_active: boolean
num_members: number
per_address_limit: number
start_time: Expiration
unit_price: Coin
[k: string]: unknown
}
