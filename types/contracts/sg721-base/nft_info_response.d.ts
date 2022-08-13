import { Empty } from "./shared-types";

export interface NftInfoResponse {
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
