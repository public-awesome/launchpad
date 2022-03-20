/**
 * A group member has a weight associated with them. This may all be equal, or may have meaning in the app that makes use of the group (eg. voting power)
 */
export interface Member {
    [k: string]: unknown;
    addr: string;
    weight: number;
}
