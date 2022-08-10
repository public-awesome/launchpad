import { RoyaltyInfoResponse } from "./shared-types";

export interface CollectionInfoResponse {
creator: string
description: string
external_link?: (string | null)
image: string
royalty_info?: (RoyaltyInfoResponse | null)
[k: string]: unknown
}
