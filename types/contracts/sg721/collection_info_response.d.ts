import { RoyaltyInfoResponse } from "./shared-types";

export interface CollectionInfoResponse {
creator: string
description: string
external_link?: (string | null)
image: string
royalty?: (RoyaltyInfoResponse | null)
[k: string]: unknown
}
