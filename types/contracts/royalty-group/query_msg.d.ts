export type QueryMsg = ({
admin: {
[k: string]: unknown
}
} | {
total_weight: {
[k: string]: unknown
}
} | {
list_members: {
limit?: (number | null)
start_after?: (string | null)
[k: string]: unknown
}
} | {
member: {
addr: string
at_height?: (number | null)
[k: string]: unknown
}
} | {
hooks: {
[k: string]: unknown
}
})
