export type QueryMsg = ({
owner_of: {
include_expired?: (boolean | null)
token_id: string
[k: string]: unknown
}
} | {
approval: {
include_expired?: (boolean | null)
spender: string
token_id: string
[k: string]: unknown
}
} | {
approvals: {
include_expired?: (boolean | null)
token_id: string
[k: string]: unknown
}
} | {
all_operators: {
include_expired?: (boolean | null)
limit?: (number | null)
owner: string
start_after?: (string | null)
[k: string]: unknown
}
} | {
num_tokens: {
[k: string]: unknown
}
} | {
contract_info: {
[k: string]: unknown
}
} | {
nft_info: {
token_id: string
[k: string]: unknown
}
} | {
all_nft_info: {
include_expired?: (boolean | null)
token_id: string
[k: string]: unknown
}
} | {
tokens: {
limit?: (number | null)
owner: string
start_after?: (string | null)
[k: string]: unknown
}
} | {
all_tokens: {
limit?: (number | null)
start_after?: (string | null)
[k: string]: unknown
}
} | {
minter: {
[k: string]: unknown
}
} | {
contract_uri: {
[k: string]: unknown
}
} | {
creator: {
[k: string]: unknown
}
} | {
royalties: {
[k: string]: unknown
}
})
