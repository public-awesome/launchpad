import { RoyaltyInfo } from "./shared-types";

export interface CollectionInfoResponse {
description: string
external_link?: (string | null)
image: string
royalty?: (RoyaltyInfo | null)
[k: string]: unknown
}
