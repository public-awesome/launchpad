export type QueryMsg = ({
start_time: {
[k: string]: unknown
}
} | {
end_time: {
[k: string]: unknown
}
} | {
has_started: {
[k: string]: unknown
}
} | {
has_ended: {
[k: string]: unknown
}
} | {
members: {
[k: string]: unknown
}
} | {
has_member: {
member: string
[k: string]: unknown
}
} | {
unit_price: {
[k: string]: unknown
}
})
