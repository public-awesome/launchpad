import { ChannelInfo } from "./shared-types";

export interface ListChannelsResponse {
channels: ChannelInfo[]
[k: string]: unknown
}
