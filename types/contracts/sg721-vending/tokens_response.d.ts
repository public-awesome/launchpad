export interface TokensResponse {
/**
 * Contains all token_ids in lexicographical ordering If there are more than `limit`, use `start_from` in future queries to achieve pagination.
 */
tokens: string[]
[k: string]: unknown
}
