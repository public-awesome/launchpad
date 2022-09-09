export type QueryMsg = ({
config: {
[k: string]: unknown
}
} | {
member: {
address: string
[k: string]: unknown
}
} | {
list_members: {
limit?: (number | null)
start_after?: (string | null)
[k: string]: unknown
}
})
