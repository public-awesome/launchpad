import { Approval } from "./shared-types";

export interface ApprovalsResponse {
approvals: Approval[]
[k: string]: unknown
}
