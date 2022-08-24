export type QueryMsg = ({
config: {
[k: string]: unknown
}
} | {
status: {
[k: string]: unknown
}
})
