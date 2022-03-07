import { Timestamp } from "./shared-types";

export type ExecuteMsg = ({
update_start_time: Timestamp
} | {
update_end_time: Timestamp
} | {
add_members: AddMembersMsg
} | {
remove_members: RemoveMembersMsg
} | {
update_per_address_limit: number
} | {
increase_member_limit: number
})

export interface AddMembersMsg {
to_add: string[]
[k: string]: unknown
}
export interface RemoveMembersMsg {
to_remove: string[]
[k: string]: unknown
}
