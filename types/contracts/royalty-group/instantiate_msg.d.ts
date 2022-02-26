import { Member } from "./shared-types";

export interface InstantiateMsg {
/**
 * The admin is the only account that can update the group state. Omit it to make the group immutable.
 */
admin?: (string | null)
members: Member[]
[k: string]: unknown
}
