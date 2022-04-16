import { Uint128 } from "./shared-types";

export type ExecuteMsg = ({
delegate: {
validator: string
[k: string]: unknown
}
} | {
undelegate: {
amount: Uint128
validator: string
[k: string]: unknown
}
} | {
redelegate: {
amount: Uint128
dst_validator: string
src_validator: string
[k: string]: unknown
}
} | {
claim: {
validator: string
[k: string]: unknown
}
})
