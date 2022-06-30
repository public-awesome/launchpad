export interface ChannelInfo {
    [k: string]: unknown;
    /**
     * the connection this exists on (you can use to query client/consensus info)
     */
    connection_id: string;
    /**
     * the remote channel/port we connect to
     */
    counterparty_endpoint: IbcEndpoint;
    /**
     * id of this channel
     */
    id: string;
}
export interface IbcEndpoint {
    [k: string]: unknown;
    channel_id: string;
    port_id: string;
}
