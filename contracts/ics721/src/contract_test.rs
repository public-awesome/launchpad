#[cfg(test)]
mod contact_test {
    use super::super::{*};
    use crate::test_helpers::*;

    use cosmwasm_std::testing::mock_env;
    use cosmwasm_std::{
        from_binary, StdError
    };
    use cosmwasm_std::{
         IbcEndpoint
    };
    use cw20_ics20::state::ChannelInfo;

    fn test_setup(deps: Deps, channel_0: String, channel_1: String) {
        // let deps = setup(&["channel-3", "channel-7"]);
        let raw_list = query(deps, mock_env(), QueryMsg::ListChannels {}).unwrap();
        let list_res: ListChannelsResponse = from_binary(&raw_list).unwrap();
        assert_eq!(2, list_res.channels.len());
        assert_eq!(mock_channel_info(&channel_0), list_res.channels[0]);
        assert_eq!(mock_channel_info(&channel_1), list_res.channels[1]);
    }

    #[test]
    fn test_query_success() {
        let deps = setup(&["channel-3", "channel-7"]);
        test_setup(
            deps.as_ref(),
            "channel-3".to_string(),
            "channel-7".to_string(),
        );

        let raw_channel = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Channel {
                id: "channel-3".to_string(),
            },
        )
        .unwrap();
        let chan_res: ChannelResponse = from_binary(&raw_channel).unwrap();
        assert_eq!(chan_res.info, mock_channel_info("channel-3"));
        assert_eq!(0, chan_res.class_ids.len());
    }
    #[test]
    fn test_query_fail() {
        let deps = setup(&["channel-3", "channel-7"]);
        test_setup(
            deps.as_ref(),
            "channel-3".to_string(),
            "channel-7".to_string(),
        );

        let err = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Channel {
                id: "channel-10".to_string(),
            },
        )
        .unwrap_err();
        assert_eq!(err, StdError::not_found("cw20_ics20::state::ChannelInfo"));
    }

    #[test]
    fn test_query_channel_list_success() {
        let deps = setup(&["channel-3", "channel-7"]);
        test_setup(
            deps.as_ref(),
            "channel-3".to_string(),
            "channel-7".to_string(),
        );

        let result = query_list(deps.as_ref());

        let expected_list: StdResult<ListChannelsResponse> = Ok(ListChannelsResponse {
            channels: vec![
                ChannelInfo {
                    id: "channel-3".to_string(),
                    counterparty_endpoint: IbcEndpoint {
                        port_id: "transfer-nft".to_string(),
                        channel_id: "channel-35".to_string(),
                    },
                    connection_id: "connection-2".to_string(),
                },
                ChannelInfo {
                    id: "channel-7".to_string(),
                    counterparty_endpoint: IbcEndpoint {
                        port_id: "transfer-nft".to_string(),
                        channel_id: "channel-75".to_string(),
                    },
                    connection_id: "connection-2".to_string(),
                },
            ],
        });
        assert_eq!(result, expected_list);
    }
    #[test]
    fn test_query_channel_list_empty() {
        let mut deps = setup(&["channel-3", "channel-7"]);
        test_setup(
            deps.as_ref(),
            "channel-3".to_string(),
            "channel-7".to_string(),
        );

        CHANNEL_INFO.remove(&mut deps.storage, "channel-3");
        CHANNEL_INFO.remove(&mut deps.storage, "channel-7");
        let result = query_list(deps.as_ref());

        let expected_list: StdResult<ListChannelsResponse> =
            Ok(ListChannelsResponse { channels: vec![] });
        assert_eq!(result, expected_list);
    }

    #[test]
    fn test_query_channel_success() {
        let mut deps = setup(&["channel-3", "channel-7"]);
        test_setup(
            deps.as_ref(),
            "channel-3".to_string(),
            "channel-7".to_string(),
        );

        let info = ChannelInfo {
            id: "channel-1".to_string(),
            counterparty_endpoint: IbcEndpoint {
                port_id: ("counterparty-port1".to_string()),
                channel_id: ("counterparty-channel-1".to_string()),
            },
            connection_id: "connection-id-1".to_string(),
        };
        CHANNEL_INFO
            .save(deps.as_mut().storage, "99", &info)
            .unwrap();

        let contract_addr = "abc/123/collection-addr";

        CHANNEL_STATE
            .save(
                deps.as_mut().storage,
                ("99", contract_addr, "1"),
                &cosmwasm_std::Empty {},
            )
            .unwrap();

        let result = query_channel(deps.as_ref(), "99".to_string());
        let expected_response = Ok(ChannelResponse {
            info: ChannelInfo {
                id: "channel-1".to_string(),
                counterparty_endpoint: IbcEndpoint {
                    port_id: "counterparty-port1".to_string(),
                    channel_id: "counterparty-channel-1".to_string(),
                },
                connection_id: "connection-id-1".to_string(),
            },
            class_ids: vec!["abc/123/collection-addr".to_string()],
        });
        assert_eq!(result, expected_response);
    }

    #[test]
    fn test_query_channel_not_found_error() {
        let mut deps = setup(&["channel-3", "channel-7"]);
        test_setup(
            deps.as_ref(),
            "channel-3".to_string(),
            "channel-7".to_string(),
        );

        let info = ChannelInfo {
            id: "channel-1".to_string(),
            counterparty_endpoint: IbcEndpoint {
                port_id: ("counterparty-port1".to_string()),
                channel_id: ("counterparty-channel-1".to_string()),
            },
            connection_id: "connection-id-1".to_string(),
        };
        CHANNEL_INFO
            .save(deps.as_mut().storage, "99", &info)
            .unwrap();

        let contract_addr = "abc/123/collection-addr";

        CHANNEL_STATE
            .save(
                deps.as_mut().storage,
                ("99", contract_addr, "1"),
                &cosmwasm_std::Empty {},
            )
            .unwrap();

        let result = query_channel(deps.as_ref(), "101".to_string());
        let expected_response = Err(StdError::NotFound {
            kind: "cw20_ics20::state::ChannelInfo".to_string(),
        });
        assert_eq!(result, expected_response);
    }

    #[test]
    fn test_query_channel_duplicates_filtered() {
        let mut deps = setup(&["channel-3", "channel-7"]);
        test_setup(
            deps.as_ref(),
            "channel-3".to_string(),
            "channel-7".to_string(),
        );

        let info = ChannelInfo {
            id: "channel-1".to_string(),
            counterparty_endpoint: IbcEndpoint {
                port_id: ("counterparty-port1".to_string()),
                channel_id: ("counterparty-channel-1".to_string()),
            },
            connection_id: "connection-id-1".to_string(),
        };
        CHANNEL_INFO
            .save(deps.as_mut().storage, "99", &info)
            .unwrap();

        let contract_addr = "abc/123/collection-addr";

        CHANNEL_STATE
            .save(
                deps.as_mut().storage,
                ("99", contract_addr, "1"),
                &cosmwasm_std::Empty {},
            )
            .unwrap();

        CHANNEL_STATE
            .save(
                deps.as_mut().storage,
                ("99", contract_addr, "2"),
                &cosmwasm_std::Empty {},
            )
            .unwrap();

        let result = query_channel(deps.as_ref(), "99".to_string());
        let expected_response = Ok(ChannelResponse {
            info: ChannelInfo {
                id: "channel-1".to_string(),
                counterparty_endpoint: IbcEndpoint {
                    port_id: "counterparty-port1".to_string(),
                    channel_id: "counterparty-channel-1".to_string(),
                },
                connection_id: "connection-id-1".to_string(),
            },
            class_ids: vec!["abc/123/collection-addr".to_string()],
        });
        assert_eq!(result, expected_response);
    }

    #[test]
    fn test_query_channel_multiple_success() {
        let mut deps = setup(&["channel-3", "channel-7"]);
        test_setup(
            deps.as_ref(),
            "channel-3".to_string(),
            "channel-7".to_string(),
        );

        let info = ChannelInfo {
            id: "channel-1".to_string(),
            counterparty_endpoint: IbcEndpoint {
                port_id: ("counterparty-port1".to_string()),
                channel_id: ("counterparty-channel-1".to_string()),
            },
            connection_id: "connection-id-1".to_string(),
        };
        CHANNEL_INFO
            .save(deps.as_mut().storage, "99", &info)
            .unwrap();

        let contract_addr = "abc/123/collection-addr";
        let contract_addr2 = "abc/456/collection-addr";

        CHANNEL_STATE
            .save(
                deps.as_mut().storage,
                ("99", contract_addr, "1"),
                &cosmwasm_std::Empty {},
            )
            .unwrap();

        CHANNEL_STATE
            .save(
                deps.as_mut().storage,
                ("99", contract_addr2, "1"),
                &cosmwasm_std::Empty {},
            )
            .unwrap();

        let result = query_channel(deps.as_ref(), "99".to_string());
        let expected_response = Ok(ChannelResponse {
            info: ChannelInfo {
                id: "channel-1".to_string(),
                counterparty_endpoint: IbcEndpoint {
                    port_id: "counterparty-port1".to_string(),
                    channel_id: "counterparty-channel-1".to_string(),
                },
                connection_id: "connection-id-1".to_string(),
            },
            class_ids: vec![
                "abc/123/collection-addr".to_string(),
                "abc/456/collection-addr".to_string(),
            ],
        });
        assert_eq!(result, expected_response);
    }
}
