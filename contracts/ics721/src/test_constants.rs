#![cfg(test)]
use crate::test_helpers::ChannelSetupData;

pub const CHANNEL_FROM_STARS_TO_OMNI: &str = "channel-3";
pub const CHANNEL_FROM_OMNI_TO_STARS: &str = "channel-7";
pub const CHANNEL_FROM_STARS_TO_GB: &str = "channel-4";
pub const CHANNEL_FROM_GB_TO_STARS: &str = "channel-8";

pub const CONNECTION_0: &str = "connection-0";
pub const CONNECTION_1: &str = "connection-1";

pub const TEST_CHANNEL_0_DATA: ChannelSetupData = ChannelSetupData {
    source_channel: CHANNEL_FROM_STARS_TO_OMNI,
    dest_channel: CHANNEL_FROM_OMNI_TO_STARS,
    connection: CONNECTION_0,
};

pub const TEST_CHANNEL_1_DATA: ChannelSetupData = ChannelSetupData {
    source_channel: CHANNEL_FROM_STARS_TO_GB,
    dest_channel: CHANNEL_FROM_GB_TO_STARS,
    connection: CONNECTION_1,
};
