import { RoyaltyInfoResponse } from "./shared-types";

export interface InstantiateMsg {
collection_info: CollectionInfoFor_RoyaltyInfoResponse
minter: string
name: string
symbol: string
[k: string]: unknown
}
export interface CollectionInfoFor_RoyaltyInfoResponse {
creator: string
description: string
external_link?: (string | null)
image: string
royalty_info?: (RoyaltyInfoResponse | null)
[k: string]: unknown
}
