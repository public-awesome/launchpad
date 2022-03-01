import { Expiration } from "./shared-types";

export type ExecuteMsg = ({
mint: {
[k: string]: unknown
}
} | {
set_whitelist: {
whitelist: string
[k: string]: unknown
}
} | {
update_start_time: Expiration
} | {
update_per_address_limit: {
per_address_limit: number
[k: string]: unknown
}
} | {
mint_to: {
recipient: string
[k: string]: unknown
}
} | {
mint_for: {
recipient: string
token_id: number
[k: string]: unknown
}
})
