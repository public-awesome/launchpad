import { Empty, OwnerOfResponse } from "./shared-types";

export interface AllNftInfoResponse {
/**
 * Who can transfer the token
 */
access: OwnerOfResponse
/**
 * Data on the token itself,
 */
info: NftInfoResponseFor_Empty
[k: string]: unknown
}

export interface NftInfoResponseFor_Empty {
/**
 * You can add any custom metadata here when you extend cw721-base
 */
extension: Empty
/**
 * Universal resource identifier for this NFT Should point to a JSON file that conforms to the ERC721 Metadata JSON Schema
 */
token_uri?: (string | null)
[k: string]: unknown
}
