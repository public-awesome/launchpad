import { RoyaltyInfo } from "./shared-types";

export interface InstantiateMsg {
collection_info: CollectionInfo
minter: string
name: string
symbol: string
[k: string]: unknown
}
export interface CollectionInfo {
description: string
external_link?: (string | null)
image: string
royalties?: (RoyaltyInfo | null)
[k: string]: unknown
}
