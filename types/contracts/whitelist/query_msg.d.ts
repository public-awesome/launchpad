export type QueryMsg = ({
has_started: {
[k: string]: unknown
}
} | {
has_ended: {
[k: string]: unknown
}
} | {
is_active: {
[k: string]: unknown
}
} | {
members: {
limit?: (number | null)
start_after?: (string | null)
[k: string]: unknown
}
} | {
has_member: {
member: string
[k: string]: unknown
}
} | {
config: {
[k: string]: unknown
}
})
