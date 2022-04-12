#[cfg(test)]
mod ibc_testing {

    use std::vec;

    use super::super::*;
    use crate::test_constants::{
        CHANNEL_FROM_OMNI_TO_STARS, CHANNEL_FROM_STARS_TO_OMNI, CONNECTION_0, TEST_CHANNEL_0_DATA,
        TEST_CHANNEL_1_DATA,
    };
    use crate::test_helpers::*;
    use cosmwasm_std::CosmosMsg::Wasm;
    use cosmwasm_std::WasmMsg::Execute;

    use crate::contract::query_channel;
    use cosmwasm_std::testing::mock_dependencies;
    use cosmwasm_std::testing::mock_env;
    use cosmwasm_std::{
        to_vec, Attribute, IbcAcknowledgement, IbcEndpoint, IbcTimeout, ReplyOn, Timestamp,
    };

    pub fn mock_sent_packet(
        my_channel: &str,
        class_id: &str,
        token_ids: Vec<&str>,
        token_uris: Vec<&str>,
        sender: &str,
    ) -> IbcPacket {
        let data = Ics721Packet {
            class_id: class_id.into(),
            class_uri: None,
            token_ids: token_ids
                .iter()
                .map(|&s| s.to_string())
                .collect::<Vec<String>>(),
            token_uris: token_uris
                .iter()
                .map(|&s| s.to_string())
                .collect::<Vec<String>>(),
            sender: sender.to_string(),
            receiver: "remote-rcpt".to_string(),
        };
        println!("Packet class_id: {}", &data.class_id);
        IbcPacket::new(
            to_binary(&data).unwrap(),
            IbcEndpoint {
                port_id: CONTRACT_PORT.to_string(),
                channel_id: my_channel.to_string(),
            },
            IbcEndpoint {
                port_id: REMOTE_PORT.to_string(),
                channel_id: "channel-1234".to_string(),
            },
            2,
            IbcTimeout::with_timestamp(Timestamp::from_seconds(1665321069)),
        )
    }

    fn mock_receive_packet(
        my_channel: &str,
        class_id: &str,
        token_ids: Vec<&str>,
        token_uris: Vec<&str>,
        receiver: &str,
    ) -> IbcPacket {
        let data = Ics721Packet {
            // this is returning a foreign (our) token, thus class_id is <port>/<channel>/<contract_addr>
            class_id: format!("{}/{}/{}", REMOTE_PORT, "channel-1234", class_id),
            class_uri: None,
            token_ids: token_ids
                .iter()
                .map(|&s| s.to_string())
                .collect::<Vec<String>>(),
            token_uris: token_uris
                .iter()
                .map(|&s| s.to_string())
                .collect::<Vec<String>>(),
            sender: "remote-sender".to_string(),
            receiver: receiver.to_string(),
        };
        println!("Packet class_id: {}", &data.class_id);
        IbcPacket::new(
            to_binary(&data).unwrap(),
            IbcEndpoint {
                port_id: REMOTE_PORT.to_string(),
                channel_id: "channel-1234".to_string(),
            },
            IbcEndpoint {
                port_id: CONTRACT_PORT.to_string(),
                channel_id: my_channel.to_string(),
            },
            3,
            Timestamp::from_seconds(1665321069).into(),
        )
    }

    fn send_sg721_success(
        deps: DepsMut,
        send_channel: String,
        contract_addr: String,
        token_ids: Vec<&str>,
        token_uris: Vec<&str>,
    ) -> IbcBasicResponse {
        // prepare some mock packets
        let sent_packet = mock_sent_packet(
            &send_channel,
            &contract_addr,
            token_ids.clone(),
            token_uris.clone(),
            "local-sender",
        );

        // we get a success cache (ack) for a send
        let msg = IbcPacketAckMsg::new(IbcAcknowledgement::new(ack_success()), sent_packet);
        ibc_packet_ack(deps, mock_env(), msg).unwrap()
    }

    fn send_sg721_fail(
        deps: DepsMut,
        send_channel: String,
        contract_addr: String,
        token_ids: Vec<&str>,
        token_uris: Vec<&str>,
    ) -> IbcBasicResponse {
        // prepare some mock packets
        let sent_packet = mock_sent_packet(
            &send_channel,
            &contract_addr,
            token_ids.clone(),
            token_uris.clone(),
            "local-sender",
        );
        // we get a fail cache (ack) for a send
        let msg = IbcPacketAckMsg::new(
            IbcAcknowledgement::new(ack_fail("Ibc Packet Fail".to_string())),
            sent_packet,
        );
        ibc_packet_ack(deps, mock_env(), msg).unwrap()
    }

    fn send_sg721_fail_res(
        deps: DepsMut,
        send_channel: String,
        contract_addr: String,
        token_ids: Vec<&str>,
        token_uris: Vec<&str>,
    ) -> Result<IbcBasicResponse, ContractError> {
        // prepare some mock packets
        let sent_packet = mock_sent_packet(
            &send_channel,
            &contract_addr,
            token_ids.clone(),
            token_uris.clone(),
            "local-sender",
        );
        // we get a fail cache (ack) for a send
        let msg = IbcPacketAckMsg::new(
            IbcAcknowledgement::new(ack_fail("Packet Fail".to_string())),
            sent_packet,
        );
        let ibc_response = ibc_packet_ack(deps, mock_env(), msg);
        match ibc_response {
            Ok(_ibc_response) => Ok(_ibc_response),
            Err(_ibc_response) => Err(ContractError::NoForeignTokens {}),
        }
    }

    fn check_query_channel_state(
        deps: DepsMut,
        send_channel: String,
        connection_id: String,
        counterparty_port_id: String,
        counterparty_channel_id: String,
    ) {
        // query channel state|_|
        let _state = query_channel(deps.as_ref(), send_channel.to_string()).unwrap();
        let channel_info = _state.info;

        let state_channel_id = channel_info.id;
        let state_counterparty_port_id = channel_info.counterparty_endpoint.port_id;
        let state_counterparty_channel_id = channel_info.counterparty_endpoint.channel_id;
        let state_connection_id = channel_info.connection_id;

        assert_eq!(state_channel_id, send_channel);
        assert_eq!(state_connection_id, connection_id);
        assert_eq!(state_counterparty_port_id, counterparty_port_id);
        assert_eq!(state_counterparty_channel_id, counterparty_channel_id);
    }

    #[test]
    fn test_ack_json() {
        let success = Ics20Ack::Result(b"1".into());
        let fail = Ics20Ack::Error("bad coin".into());

        let success_json = String::from_utf8(to_vec(&success).unwrap()).unwrap();
        assert_eq!(r#"{"result":"MQ=="}"#, success_json.as_str());

        let fail_json = String::from_utf8(to_vec(&fail).unwrap()).unwrap();
        assert_eq!(r#"{"error":"bad coin"}"#, fail_json.as_str());
    }

    #[test]
    fn test_packet_json() {
        let packet = Ics721Packet::new(
            "stars1zedxv25ah8fksmg2lzrndrpkvsjqgk4zt5ff7n",
            Some("https://metadata-url.com/my-metadata"),
            vec!["1", "2", "3"],
            vec![
                "https://metadata-url.com/my-metadata1",
                "https://metadata-url.com/my-metadata2",
                "https://metadata-url.com/my-metadata3",
            ],
            "stars1zedxv25ah8fksmg2lzrndrpkvsjqgk4zt5ff7n",
            "wasm1fucynrfkrt684pm8jrt8la5h2csvs5cnldcgqc",
        );
        // Example message generated from the SDK
        let expected = r#"{"class_id":"stars1zedxv25ah8fksmg2lzrndrpkvsjqgk4zt5ff7n","class_uri":"https://metadata-url.com/my-metadata","token_ids":["1","2","3"],"token_uris":["https://metadata-url.com/my-metadata1","https://metadata-url.com/my-metadata2","https://metadata-url.com/my-metadata3"],"sender":"stars1zedxv25ah8fksmg2lzrndrpkvsjqgk4zt5ff7n","receiver":"wasm1fucynrfkrt684pm8jrt8la5h2csvs5cnldcgqc"}"#;

        let encdoded = String::from_utf8(to_vec(&packet).unwrap()).unwrap();
        assert_eq!(expected, encdoded.as_str());
    }

    fn _cw721_transfer(token_id: String, address: &str, recipient: &str) -> SubMsg {
        let msg = Cw721ExecuteMsg::TransferNft {
            token_id,
            recipient: recipient.into(),
        };
        let exec = WasmMsg::Execute {
            contract_addr: address.into(),
            msg: to_binary(&msg).unwrap(),
            funds: vec![],
        };
        SubMsg::reply_on_error(exec, SEND_NFT_ID)
    }

    #[test]
    fn test_query_channel() {
        let send_channel = CHANNEL_FROM_STARS_TO_OMNI;
        let mut deps = setup(&[TEST_CHANNEL_0_DATA, TEST_CHANNEL_1_DATA]);
        let connection_id = CONNECTION_0;
        let counterparty_port_id = REMOTE_PORT;
        let counterparty_channel_id = CHANNEL_FROM_OMNI_TO_STARS;
        check_query_channel_state(
            deps.as_mut(),
            send_channel.to_string(),
            connection_id.to_string(),
            counterparty_port_id.to_string(),
            counterparty_channel_id.to_string(),
        );
    }

    #[test]
    fn test_receive_sg721_multiple_success() {
        let send_channel = CHANNEL_FROM_STARS_TO_OMNI;
        let mut deps = setup(&[TEST_CHANNEL_0_DATA, TEST_CHANNEL_1_DATA]);
        let contract_addr = "collection-addr";
        let token_ids = vec!["1", "2", "3"];
        let token_uris = vec![
            "https://metadata-url.com/my-metadata1",
            "https://metadata-url.com/my-metadata2",
            "https://metadata-url.com/my-metadata3",
        ];

        // before tokens are sent, they are not on the channel state
        let exists = CHANNEL_STATE.may_load(&deps.storage, (send_channel, contract_addr, "1"));
        assert_eq!(exists, Ok(None));
        let exists = CHANNEL_STATE.may_load(&deps.storage, (send_channel, contract_addr, "2"));
        assert_eq!(exists, Ok(None));
        let exists = CHANNEL_STATE.may_load(&deps.storage, (send_channel, contract_addr, "3"));
        assert_eq!(exists, Ok(None));

        send_sg721_success(
            deps.as_mut(),
            send_channel.to_string(),
            contract_addr.to_string(),
            token_ids.clone(),
            token_uris.clone(),
        );

        // channel state now has 3 token ids
        let exists = CHANNEL_STATE.may_load(&deps.storage, (send_channel, contract_addr, "1"));
        assert_eq!(exists, Ok(Some(Empty {})));
        let exists = CHANNEL_STATE.may_load(&deps.storage, (send_channel, contract_addr, "2"));
        assert_eq!(exists, Ok(Some(Empty {})));
        let exists = CHANNEL_STATE.may_load(&deps.storage, (send_channel, contract_addr, "3"));
        assert_eq!(exists, Ok(Some(Empty {})));

        let recv_packet = mock_receive_packet(
            send_channel,
            contract_addr,
            token_ids,
            token_uris,
            "local-rcpt",
        );

        let packet_receive = IbcPacketReceiveMsg::new(recv_packet);
        let res = ibc_packet_receive(deps.as_mut(), mock_env(), packet_receive).unwrap();

        // after receive token ids 1,2, and 3 are now removed from channel state
        let exists = CHANNEL_STATE.may_load(&deps.storage, (send_channel, contract_addr, "1"));
        assert_eq!(exists, Ok(None));
        let exists = CHANNEL_STATE.may_load(&deps.storage, (send_channel, contract_addr, "2"));
        assert_eq!(exists, Ok(None));
        let exists = CHANNEL_STATE.may_load(&deps.storage, (send_channel, contract_addr, "3"));
        assert_eq!(exists, Ok(None));

        let cw721_execute_msgs = [
            Cw721ExecuteMsg::TransferNft {
                recipient: "local-rcpt".into(),
                token_id: "1".into(),
            },
            Cw721ExecuteMsg::TransferNft {
                recipient: "local-rcpt".into(),
                token_id: "2".into(),
            },
            Cw721ExecuteMsg::TransferNft {
                recipient: "local-rcpt".into(),
                token_id: "3".into(),
            },
        ];

        let expected_return: SubMsg = SubMsg {
            id: 1338,
            msg: Wasm(Execute {
                contract_addr: "collection-addr".into(),
                msg: to_binary(&cw721_execute_msgs).unwrap(),
                funds: [].into(),
            }),
            gas_limit: None,
            reply_on: ReplyOn::Error,
        };
        assert_eq!(res.messages[0], expected_return);

        let res_attributes = [
            Attribute {
                key: "action".to_string(),
                value: "receive".to_string(),
            },
            Attribute {
                key: "sender".to_string(),

                value: "remote-sender".to_string(),
            },
            Attribute {
                key: "receiver".to_string(),
                value: "local-rcpt".to_string(),
            },
            Attribute {
                key: "contract_address".to_string(),
                value: "collection-addr".to_string(),
            },
            Attribute {
                key: "token_ids".to_string(),
                value: "1,2,3".to_string(),
            },
            Attribute {
                key: "success".to_string(),
                value: "true".to_string(),
            },
        ];

        assert_eq!(res.attributes, res_attributes);
    }

    #[test]
    fn test_receive_sg721_single_success() {
        let send_channel = CHANNEL_FROM_STARS_TO_OMNI;
        let mut deps = setup(&[TEST_CHANNEL_0_DATA, TEST_CHANNEL_1_DATA]);
        let contract_addr = "collection-addr";
        let token_ids = vec!["1"];
        let token_uris = vec![
            "https://metadata-url.com/my-metadata1",
            "https://metadata-url.com/my-metadata2",
            "https://metadata-url.com/my-metadata3",
        ];

        // before tokens are sent, they are not on the channel state
        let exists = CHANNEL_STATE.may_load(&deps.storage, (send_channel, contract_addr, "1"));
        assert_eq!(exists, Ok(None));

        send_sg721_success(
            deps.as_mut(),
            send_channel.to_string(),
            contract_addr.to_string(),
            token_ids.clone(),
            token_uris.clone(),
        );

        // channel state now has 1 token id
        let exists = CHANNEL_STATE.may_load(&deps.storage, (send_channel, contract_addr, "1"));
        assert_eq!(exists, Ok(Some(Empty {})));

        let recv_packet = mock_receive_packet(
            send_channel,
            contract_addr,
            token_ids,
            token_uris,
            "local-rcpt",
        );

        let packet_receive = IbcPacketReceiveMsg::new(recv_packet);
        let res = ibc_packet_receive(deps.as_mut(), mock_env(), packet_receive).unwrap();

        // after receive token id 1 is now removed from channel state
        let exists = CHANNEL_STATE.may_load(&deps.storage, (send_channel, contract_addr, "1"));
        assert_eq!(exists, Ok(None));

        let cw721_execute_msgs = [Cw721ExecuteMsg::TransferNft {
            recipient: "local-rcpt".into(),
            token_id: "1".into(),
        }];

        let expected_return: SubMsg = SubMsg {
            id: 1338,
            msg: Wasm(Execute {
                contract_addr: "collection-addr".into(),
                msg: to_binary(&cw721_execute_msgs).unwrap(),
                funds: [].into(),
            }),
            gas_limit: None,
            reply_on: ReplyOn::Error,
        };
        assert_eq!(res.messages[0], expected_return);

        let res_attributes = [
            Attribute {
                key: "action".to_string(),
                value: "receive".to_string(),
            },
            Attribute {
                key: "sender".to_string(),

                value: "remote-sender".to_string(),
            },
            Attribute {
                key: "receiver".to_string(),
                value: "local-rcpt".to_string(),
            },
            Attribute {
                key: "contract_address".to_string(),
                value: "collection-addr".to_string(),
            },
            Attribute {
                key: "token_ids".to_string(),
                value: "1".to_string(),
            },
            Attribute {
                key: "success".to_string(),
                value: "true".to_string(),
            },
        ];

        assert_eq!(res.attributes, res_attributes);
    }

    #[test]
    fn test_receive_sg721_empty() {
        let send_channel = CHANNEL_FROM_STARS_TO_OMNI;
        let mut deps = setup(&[TEST_CHANNEL_0_DATA, TEST_CHANNEL_1_DATA]);

        let contract_addr = "collection-addr";
        let token_ids = vec!["1", "2", "3"];
        let token_uris = vec![
            "https://metadata-url.com/my-metadata1",
            "https://metadata-url.com/my-metadata2",
            "https://metadata-url.com/my-metadata3",
        ];

        let recv_packet = mock_receive_packet(
            send_channel,
            contract_addr,
            token_ids,
            token_uris,
            "local-rcpt",
        );

        let msg = IbcPacketReceiveMsg::new(recv_packet);
        // cannot receive this class_id yet
        // TODO: but should be able to after implementing sending to other sg721 contracts
        let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();

        let result_attributes = [
            Attribute {
                key: "action".to_string(),
                value: "receive".to_string(),
            },
            Attribute {
                key: "success".to_string(),
                value: "false".to_string(),
            },
            Attribute {
                key: "error".to_string(),
                value: "NoSuchNft".to_string(),
            },
        ];
        assert_eq!(res.attributes, result_attributes);

        assert!(res.messages.is_empty());
        assert!(res.events.is_empty());

        let ack: Ics20Ack = from_binary(&res.acknowledgement).unwrap();
        let no_such_nft = Ics20Ack::Error(
            ContractError::NoSuchNft {
                class_id: "class_id".to_string(),
            }
            .to_string(),
        );
        assert_eq!(ack, no_such_nft);
    }

    #[test]
    fn test_send_sg721_success() {
        let send_channel = CHANNEL_FROM_STARS_TO_OMNI;
        let mut deps = setup(&[TEST_CHANNEL_0_DATA, TEST_CHANNEL_1_DATA]);
        let contract_addr = "collection-addr";
        let token_ids = vec!["1", "2", "3"];
        let token_uris = vec![
            "https://metadata-url.com/my-metadata1",
            "https://metadata-url.com/my-metadata2",
            "https://metadata-url.com/my-metadata3",
        ];

        let res = send_sg721_success(
            deps.as_mut(),
            send_channel.to_string(),
            contract_addr.to_string(),
            token_ids.clone(),
            token_uris.clone(),
        );

        let ibc_expected_response = IbcBasicResponse::new().add_attributes(
            [
                Attribute {
                    key: "action".to_string(),
                    value: "acknowledge".to_string(),
                },
                Attribute {
                    key: "sender".to_string(),
                    value: "local-sender".to_string(),
                },
                Attribute {
                    key: "receiver".to_string(),
                    value: "remote-rcpt".to_string(),
                },
                Attribute {
                    key: "contract_addr".to_string(),
                    value: "collection-addr".to_string(),
                },
                Attribute {
                    key: "success".to_string(),
                    value: "true".to_string(),
                },
            ]
            .to_vec(),
        );

        assert_eq!(res, ibc_expected_response);
    }

    #[test]
    fn test_send_sg721_fail_ibc_packet() {
        let send_channel = CHANNEL_FROM_STARS_TO_OMNI;
        let mut deps = setup(&[TEST_CHANNEL_0_DATA, TEST_CHANNEL_1_DATA]);
        let contract_addr = "transfer-nft/abc/def";
        let token_ids = vec!["1", "2", "3"];
        let token_uris = vec![
            "https://metadata-url.com/my-metadata1",
            "https://metadata-url.com/my-metadata2",
            "https://metadata-url.com/my-metadata3",
        ];

        let ibc_packet = mock_sent_packet(
            send_channel,
            contract_addr,
            token_ids.clone(),
            token_uris.clone(),
            "local-sender",
        );

        let nft_name = "my-nft";
        let mut contract_addr = ibc_packet.src.port_id.to_string();
        contract_addr += "/";
        contract_addr += &ibc_packet.src.channel_id;
        contract_addr += "/";
        contract_addr += nft_name;

        let res = send_sg721_fail(
            deps.as_mut(),
            send_channel.to_string(),
            contract_addr.clone(),
            token_ids.clone(),
            token_uris.clone(),
        );

        let expected_cw721_execute_msgs = [
            Cw721ExecuteMsg::TransferNft {
                recipient: "local-sender".into(),
                token_id: "1".into(),
            },
            Cw721ExecuteMsg::TransferNft {
                recipient: "local-sender".into(),
                token_id: "2".into(),
            },
            Cw721ExecuteMsg::TransferNft {
                recipient: "local-sender".into(),
                token_id: "3".into(),
            },
        ];

        let wasm_msg = WasmMsg::Execute {
            contract_addr: nft_name.into(),
            msg: to_binary(&expected_cw721_execute_msgs).unwrap(),
            funds: vec![],
        };
        let expected_sub_msg = SubMsg::reply_on_error(wasm_msg, SEND_NFT_ID);
        assert_eq!(res.messages[0], expected_sub_msg);

        let expoected_attributes = [
            Attribute {
                key: "action".to_string(),
                value: "acknowledge".to_string(),
            },
            Attribute {
                key: "sender".to_string(),
                value: "local-sender".to_string(),
            },
            Attribute {
                key: "receiver".to_string(),
                value: "remote-rcpt".to_string(),
            },
            Attribute {
                key: "contract_addr".to_string(),
                value: "my-nft".to_string(),
            },
            Attribute {
                key: "success".to_string(),
                value: "false".to_string(),
            },
            Attribute {
                key: "error".to_string(),
                value: "Ibc Packet Fail".to_string(),
            },
        ];
        assert_eq!(res.attributes, expoected_attributes);
    }

    #[test]
    fn test_send_sg721_fail_foreign_token() {
        let send_channel = CHANNEL_FROM_STARS_TO_OMNI;
        let mut deps = setup(&[TEST_CHANNEL_0_DATA, TEST_CHANNEL_1_DATA]);
        let contract_addr = "transfer-nft/abc/def";
        let token_ids = vec!["1", "2", "3"];
        let token_uris = vec![
            "https://metadata-url.com/my-metadata1",
            "https://metadata-url.com/my-metadata2",
            "https://metadata-url.com/my-metadata3",
        ];

        let _ibc_packet = mock_sent_packet(
            send_channel,
            contract_addr,
            token_ids.clone(),
            token_uris.clone(),
            "local-sender",
        );

        let res = send_sg721_fail_res(
            deps.as_mut(),
            send_channel.to_string(),
            contract_addr.to_string(),
            token_ids.clone(),
            token_uris.clone(),
        );

        let error_str: String = res.unwrap_err().to_string();
        let expected_error_msg =
            "Only accepts tokens that originate on this chain, not native tokens of remote chain";
        assert_eq!(error_str, expected_error_msg);
    }

    #[test]
    fn test_parse_voucher_contract_address_success() {
        let send_channel = CHANNEL_FROM_STARS_TO_OMNI;
        let contract_port = "ibc:wasm1234567890abcdef";

        let endpoint_1 = IbcEndpoint {
            port_id: contract_port.to_string(),
            channel_id: send_channel.to_string(),
        };

        let voucher_class_id = &format!(
            "ibc:wasm1234567890abcdef/{}/my-nft",
            CHANNEL_FROM_STARS_TO_OMNI
        );
        let parse_result = parse_voucher_contract_address(voucher_class_id, &endpoint_1);
        assert_eq!(parse_result.unwrap().to_string(), "my-nft");
    }

    #[test]
    fn test_parse_voucher_contract_address_fail_other_port() {
        let send_channel = CHANNEL_FROM_STARS_TO_OMNI;
        let contract_port = "ibc:wasm1234567890abcdef";

        let endpoint_1 = IbcEndpoint {
            port_id: contract_port.to_string(),
            channel_id: send_channel.to_string(),
        };

        let voucher_class_id = &format!("other-port/{}/my-nft", CHANNEL_FROM_STARS_TO_OMNI);
        let parse_result = parse_voucher_contract_address(voucher_class_id, &endpoint_1);

        let error_msg = parse_result.unwrap_err().to_string();
        assert_eq!(
            error_msg,
            "Parsed port from denom (other-port) doesn't match packet"
        );
    }

    #[test]
    fn test_parse_voucher_contract_address_fail_other_channel() {
        let send_channel = CHANNEL_FROM_STARS_TO_OMNI;
        let contract_port = "ibc:wasm1234567890abcdef";

        let endpoint_1 = IbcEndpoint {
            port_id: contract_port.to_string(),
            channel_id: send_channel.to_string(),
        };

        let voucher_class_id = "ibc:wasm1234567890abcdef/other-channel/my-nft";
        let parse_result = parse_voucher_contract_address(voucher_class_id, &endpoint_1);

        let error_msg = parse_result.unwrap_err().to_string();
        assert_eq!(
            error_msg,
            "Parsed channel from denom (other-channel) doesn't match packet"
        );
    }

    #[test]
    fn test_enforce_order_and_version_success() {
        let send_channel = CHANNEL_FROM_STARS_TO_OMNI;
        let counterparty_send_channel = CHANNEL_FROM_OMNI_TO_STARS;
        let counterparty_contract_port = "ibc:stars123abc";
        let contract_port = "ibc:wasm1234567890abcdef";

        let endpoint_1 = IbcEndpoint {
            port_id: contract_port.to_string(),
            channel_id: send_channel.to_string(),
        };

        let endpoint_2 = IbcEndpoint {
            port_id: counterparty_contract_port.to_string(),
            channel_id: counterparty_send_channel.to_string(),
        };

        let ibc_channel = cosmwasm_std::IbcChannel::new(
            endpoint_1,
            endpoint_2,
            IbcOrder::Unordered,
            ICS721_VERSION,
            CONNECTION_0.to_string(),
        );
        let result = enforce_order_and_version(&ibc_channel, Some(ICS721_VERSION));
        match result {
            Ok(_val) => (),
            Err(e) => {
                panic!("enforce_order_and_version returned error {:?}", e);
            }
        }
    }

    #[test]
    fn test_enforce_order_and_version_ibc_channel_wrong_version_fail() {
        let send_channel = CHANNEL_FROM_STARS_TO_OMNI;
        let counterparty_send_channel = CHANNEL_FROM_OMNI_TO_STARS;
        let counterparty_contract_port = "ibc:stars123abc";
        let contract_port = "ibc:wasm1234567890abcdef";

        let endpoint_1 = IbcEndpoint {
            port_id: contract_port.to_string(),
            channel_id: send_channel.to_string(),
        };

        let endpoint_2 = IbcEndpoint {
            port_id: counterparty_contract_port.to_string(),
            channel_id: counterparty_send_channel.to_string(),
        };

        let ibc_channel = cosmwasm_std::IbcChannel::new(
            endpoint_1,
            endpoint_2,
            IbcOrder::Unordered,
            "very_fake_version",
            CONNECTION_0.to_string(),
        );

        let result = enforce_order_and_version(&ibc_channel, Some(ICS721_VERSION));
        assert_eq!(
            result.unwrap_err().to_string(),
            "Only supports channel with ibc version ics721-1, got very_fake_version"
        );
    }

    #[test]
    fn test_enforce_order_and_version_counterparty_version_wrong_version_fail() {
        let send_channel = CHANNEL_FROM_STARS_TO_OMNI;
        let counterparty_send_channel = CHANNEL_FROM_OMNI_TO_STARS;
        let counterparty_contract_port = "ibc:stars123abc";
        let contract_port = "ibc:wasm1234567890abcdef";

        let endpoint_1 = IbcEndpoint {
            port_id: contract_port.to_string(),
            channel_id: send_channel.to_string(),
        };

        let endpoint_2 = IbcEndpoint {
            port_id: counterparty_contract_port.to_string(),
            channel_id: counterparty_send_channel.to_string(),
        };

        let ibc_channel = cosmwasm_std::IbcChannel::new(
            endpoint_1,
            endpoint_2,
            IbcOrder::Unordered,
            ICS721_VERSION,
            CONNECTION_0.to_string(),
        );

        let result =
            enforce_order_and_version(&ibc_channel, Some("very_fake_version_counterparty"));
        assert_eq!(
            result.unwrap_err().to_string(),
            "Only supports channel with ibc version ics721-1, got very_fake_version_counterparty"
        );
    }

    #[test]
    fn test_channel_connect() {
        let send_channel = CHANNEL_FROM_STARS_TO_OMNI;
        let counterparty_send_channel = CHANNEL_FROM_OMNI_TO_STARS;
        let counterparty_contract_port = "ibc:stars123abc";
        let contract_port = "ibc:wasm1234567890abcdef";

        let mut deps = mock_dependencies();

        let endpoint_1 = IbcEndpoint {
            port_id: contract_port.to_string(),
            channel_id: send_channel.to_string(),
        };

        let endpoint_2 = IbcEndpoint {
            port_id: counterparty_contract_port.to_string(),
            channel_id: counterparty_send_channel.to_string(),
        };

        let ibc_channel = cosmwasm_std::IbcChannel::new(
            endpoint_1,
            endpoint_2,
            IbcOrder::Unordered,
            ICS721_VERSION,
            CONNECTION_0.to_string(),
        );

        let channel_connect_msg = IbcChannelConnectMsg::OpenAck {
            channel: (ibc_channel),
            counterparty_version: (ICS721_VERSION.to_string()),
        };

        let channel_info_data = CHANNEL_INFO.may_load(&deps.storage, send_channel);
        assert_eq!(channel_info_data.unwrap(), None);

        let result = ibc_channel_connect(deps.as_mut(), mock_env(), channel_connect_msg);
        assert_eq!(result.unwrap(), IbcBasicResponse::default());

        let channel_info_data = CHANNEL_INFO.may_load(&deps.storage, send_channel);
        let expected_channel_data = ChannelInfo {
            id: CHANNEL_FROM_STARS_TO_OMNI.into(),
            counterparty_endpoint: IbcEndpoint {
                port_id: "ibc:stars123abc".into(),
                channel_id: CHANNEL_FROM_OMNI_TO_STARS.into(),
            },
            connection_id: CONNECTION_0.into(),
        };
        assert_eq!(channel_info_data.unwrap().unwrap(), expected_channel_data);
    }

    #[test]
    fn test_send_tokens_single() {
        let send_channel = CHANNEL_FROM_STARS_TO_OMNI;
        let mut deps = setup(&[TEST_CHANNEL_0_DATA, TEST_CHANNEL_1_DATA]);
        let contract_addr = "collection-addr";
        let token_ids = vec!["1", "2", "3"];
        let token_uris = vec![
            "https://metadata-url.com/my-metadata1",
            "https://metadata-url.com/my-metadata2",
            "https://metadata-url.com/my-metadata3",
        ];

        send_sg721_success(
            deps.as_mut(),
            send_channel.to_string(),
            contract_addr.to_string(),
            token_ids.clone(),
            token_uris.clone(),
        );

        let exists = CHANNEL_STATE.may_load(&deps.storage, (send_channel, contract_addr, "1"));
        assert_eq!(exists, Ok(Some(Empty {})));

        let result = send_tokens(
            contract_addr,
            vec!["1".into()],
            vec![
                "https://metadata-url.com/my-metadata1".into(),
                "https://metadata-url.com/my-metadata2".into(),
                "https://metadata-url.com/my-metadata3".into(),
            ],
            "local-rcpt".into(),
        );

        let cw721_msg_1 = Cw721ExecuteMsg::TransferNft {
            recipient: "local-rcpt".into(),
            token_id: "1".into(),
        };
        let msgs: Vec<Cw721ExecuteMsg> = vec![cw721_msg_1];
        let submsg: cosmwasm_std::SubMsg<Empty> = SubMsg {
            id: SEND_NFT_ID,
            msg: Wasm(Execute {
                contract_addr: "collection-addr".into(),
                msg: to_binary(&msgs).unwrap(),
                funds: vec![],
            }),
            gas_limit: None,
            reply_on: cosmwasm_std::ReplyOn::Error,
        };
        assert_eq!(result, submsg);
    }

    #[test]
    fn test_send_tokens_multiple() {
        let send_channel = CHANNEL_FROM_STARS_TO_OMNI;
        let mut deps = setup(&[TEST_CHANNEL_0_DATA, TEST_CHANNEL_1_DATA]);
        let contract_addr = "collection-addr";
        let token_ids = vec!["1", "2", "3"];
        let token_uris = vec![
            "https://metadata-url.com/my-metadata1",
            "https://metadata-url.com/my-metadata2",
            "https://metadata-url.com/my-metadata3",
        ];

        send_sg721_success(
            deps.as_mut(),
            send_channel.to_string(),
            contract_addr.to_string(),
            token_ids.clone(),
            token_uris.clone(),
        );

        let exists = CHANNEL_STATE.may_load(&deps.storage, (send_channel, contract_addr, "1"));
        assert_eq!(exists, Ok(Some(Empty {})));

        let result = send_tokens(
            contract_addr,
            vec!["1".into(), "2".into(), "3".into()],
            vec![
                "https://metadata-url.com/my-metadata1".into(),
                "https://metadata-url.com/my-metadata2".into(),
                "https://metadata-url.com/my-metadata3".into(),
            ],
            "local-rcpt".into(),
        );

        let cw721_msg_1 = Cw721ExecuteMsg::TransferNft {
            recipient: "local-rcpt".into(),
            token_id: "1".into(),
        };
        let cw721_msg_2 = Cw721ExecuteMsg::TransferNft {
            recipient: "local-rcpt".into(),
            token_id: "2".into(),
        };
        let cw721_msg_3 = Cw721ExecuteMsg::TransferNft {
            recipient: "local-rcpt".into(),
            token_id: "3".into(),
        };
        let msgs: Vec<Cw721ExecuteMsg> = vec![cw721_msg_1, cw721_msg_2, cw721_msg_3];
        let submsg: cosmwasm_std::SubMsg<Empty> = SubMsg {
            id: SEND_NFT_ID,
            msg: Wasm(Execute {
                contract_addr: "collection-addr".into(),
                msg: to_binary(&msgs).unwrap(),
                funds: vec![],
            }),
            gas_limit: None,
            reply_on: cosmwasm_std::ReplyOn::Error,
        };
        assert_eq!(result, submsg);
    }
}
