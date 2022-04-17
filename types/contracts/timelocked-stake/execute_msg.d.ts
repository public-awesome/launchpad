export type ExecuteMsg = ({
undelegate: {
[k: string]: unknown
}
} | {
redelegate: {
dst_validator: string
[k: string]: unknown
}
} | {
claim: {
[k: string]: unknown
}
} | {
reinvest: {
[k: string]: unknown
}
} | {
__delegate: {
[k: string]: unknown
}
})
