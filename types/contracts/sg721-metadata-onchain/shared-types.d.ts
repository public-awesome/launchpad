/**
 * Expiration represents a point in time when some event happens. It can compare with a BlockInfo and will return is_expired() == true once the condition is hit (and for every block in the future)
 */
export type Expiration = ({
    at_height: number
    } | {
    at_time: Timestamp
    } | {
    never: {
    [k: string]: unknown
    }
    });
/**
 * A point in time in nanosecond precision.
 *
 * This type can represent times from 1970-01-01T00:00:00Z to 2554-07-21T23:34:33Z.
 *
 * ## Examples
 *
 * ``` # use cosmwasm_std::Timestamp; let ts = Timestamp::from_nanos(1_000_000_202); assert_eq!(ts.nanos(), 1_000_000_202); assert_eq!(ts.seconds(), 1); assert_eq!(ts.subsec_nanos(), 202);
 *
 * let ts = ts.plus_seconds(2); assert_eq!(ts.nanos(), 3_000_000_202); assert_eq!(ts.seconds(), 3); assert_eq!(ts.subsec_nanos(), 202); ```
 */
export type Timestamp = Uint64;
/**
 * A thin wrapper around u64 that is using strings for JSON encoding/decoding, such that the full u64 range can be used for clients that convert JSON numbers to floats, like JavaScript and jq.
 *
 * # Examples
 *
 * Use `from` to create instances of this and `u64` to get the value out:
 *
 * ``` # use cosmwasm_std::Uint64; let a = Uint64::from(42u64); assert_eq!(a.u64(), 42);
 *
 * let b = Uint64::from(70u32); assert_eq!(b.u64(), 70); ```
 */
export type Uint64 = string;
export interface OwnerOfResponse {
    [k: string]: unknown;
    /**
     * If set this address is approved to transfer/send the token as well
     */
    approvals: Approval[];
    /**
     * Owner of the token
     */
    owner: string;
}
export interface Approval {
    [k: string]: unknown;
    /**
     * When the Approval expires (maybe Expiration::never)
     */
    expires: Expiration;
    /**
     * Account that can transfer/send the token
     */
    spender: string;
}
/**
 * OpenSea metadata standard, used by Stargaze marketplace. See [this link](https://docs.opensea.io/docs/metadata-standards) for details.
 */
export interface Metadata {
    [k: string]: unknown;
    /**
     * A URL to a multi-media attachment for the item. The file extensions GLTF, GLB, WEBM, MP4, M4V, OGV, and OGG are supported, along with the audio-only extensions MP3, WAV, and OGA.
     *
     * Animation_url also supports HTML pages, allowing you to build rich experiences and interactive NFTs using JavaScript canvas, WebGL, and more. Scripts and relative paths within the HTML page are now supported. However, access to browser extensions is not supported.
     */
    animation_url?: (string | null);
    /**
     * These are the attributes for the item, which will show up on the OpenSea page for the item.
     */
    attributes?: (Trait[] | null);
    /**
     * Background color of the item on OpenSea. Must be a six-character hexadecimal without a pre-pended #.
     */
    background_color?: (string | null);
    /**
     * A human readable description of the item. Markdown is supported.
     */
    description?: (string | null);
    /**
     * This is the URL that will appear below the asset's image on OpenSea and will allow users to leave OpenSea and view the item on your site.
     */
    external_url?: (string | null);
    /**
     * This is the URL to the image of the item. Can be just about any type of image (including SVGs, which will be cached into PNGs by OpenSea), and can be [IPFS](https://github.com/ipfs/is-ipfs) URLs or paths. We recommend using a 350 x 350 image.
     */
    image?: (string | null);
    /**
     * Raw SVG image data, if you want to generate images on the fly (not recommended). Only use this if you're not including the `image` parameter.
     */
    image_data?: (string | null);
    /**
     * Name of the item.
     */
    name?: (string | null);
    /**
     * A URL to a YouTube video.
     */
    youtube_url?: (string | null);
}
/**
 * An attribute of the token as defined by the [OpenSea metadata standard](https://docs.opensea.io/docs/metadata-standards#attributes).
 */
export interface Trait {
    [k: string]: unknown;
    display_type?: (string | null);
    trait_type: string;
    value: string;
}
