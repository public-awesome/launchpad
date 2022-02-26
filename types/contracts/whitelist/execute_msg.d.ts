import { Expiration } from "./shared-types";

export type ExecuteMsg = ({
update_start_time: Expiration
} | {
update_end_time: Expiration
} | {
update_members: UpdateMembersMsg
})

export interface UpdateMembersMsg {
add: string[]
remove: string[]
[k: string]: unknown
}
