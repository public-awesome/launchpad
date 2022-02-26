import { Expiration } from "./shared-types";

export interface InstantiateMsg {
end_time: Expiration
members: string[]
start_time: Expiration
[k: string]: unknown
}
