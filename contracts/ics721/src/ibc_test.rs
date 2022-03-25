#[cfg(test)]
mod ibc_test   {
    use super::super::{*};
    use crate::test_helpers::*;
    
    use crate::contract::query_channel;
    use cosmwasm_std::testing::mock_env;
    use cosmwasm_std::{to_vec, IbcAcknowledgement, IbcEndpoint, IbcTimeout, Timestamp, Attribute, ReplyOn};


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


    fn send_sg721_success(deps: DepsMut, send_channel: String, contract_addr: String,
        token_ids: Vec<&str>, token_uris: Vec<&str>) -> IbcBasicResponse {

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
        let res = ibc_packet_ack(deps, mock_env(), msg).unwrap();
        return res
    }

    fn send_sg721_fail(deps: DepsMut, send_channel: String, contract_addr: String,
        token_ids: Vec<&str>, token_uris: Vec<&str>) -> IbcBasicResponse {

        // prepare some mock packets
        let sent_packet = mock_sent_packet(
            &send_channel,
            &contract_addr,
            token_ids.clone(),
            token_uris.clone(),
            "local-sender",
        );
        // we get a fail cache (ack) for a send
        let msg = IbcPacketAckMsg::new(IbcAcknowledgement::new(ack_fail("Packet Fail".to_string())), sent_packet);
        let res = ibc_packet_ack(deps, mock_env(), msg).unwrap();
        return res
    }


    fn check_query_channel_state(deps: DepsMut, send_channel: String, connection_id: String, 
        counterparty_port_id: String, counterparty_channel_id: String) {
            // query channel state|_|
            let _state = query_channel(deps.as_ref(), send_channel.to_string()).unwrap(); 
            let channel_info = _state.info;
    
            let state_channel_id = channel_info.id;
            let state_counterparty_port_id = channel_info.counterparty_endpoint.port_id;
            let state_counterparty_channel_id = channel_info.counterparty_endpoint.channel_id;
            let state_connection_id = channel_info.connection_id;
    
            assert_eq!(state_channel_id, send_channel.to_string());
            assert_eq!(state_connection_id, connection_id.to_string());
            assert_eq!(state_counterparty_port_id, counterparty_port_id.to_string());
            assert_eq!(state_counterparty_channel_id, counterparty_channel_id.to_string());

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
    fn test_receive_sg721_success() {
        let send_channel = "channel-9";
        let mut deps = setup(&["channel-1", "channel-7", send_channel]);
        let contract_addr = "collection-addr";
        let token_ids = vec!["1", "2", "3"];
        let token_uris = vec![
            "https://metadata-url.com/my-metadata1",
            "https://metadata-url.com/my-metadata2",
            "https://metadata-url.com/my-metadata3",
        ];

        send_sg721_success(deps.as_mut(), send_channel.to_string(), 
            contract_addr.to_string(), token_ids.clone(), token_uris.clone());

        let recv_packet = mock_receive_packet(
            send_channel,
            contract_addr,
            token_ids,
            token_uris,
            "local-rcpt",
        );

        let packet_receive = IbcPacketReceiveMsg::new(recv_packet.clone());
        let res = ibc_packet_receive(deps.as_mut(), mock_env(), packet_receive).unwrap();
        
        let res_attributes = [
            Attribute { key: "action".to_string(), value: "receive".to_string() }, 
            Attribute { key: "sender".to_string(), value: "remote-sender".to_string() },
            Attribute { key: "receiver".to_string(), value: "local-rcpt".to_string() }, 
            Attribute { key: "contract_address".to_string(), value: "collection-addr".to_string() },
            Attribute { key: "token_ids".to_string(), value: "1,2,3".to_string() },
            Attribute { key: "success".to_string(), value: "true".to_string() }];

        assert_eq!(res.attributes, res_attributes);

        let connection_id = "connection-2";
        let counterparty_port_id = "transfer-nft";
        let counterparty_channel_id = "channel-95";
        check_query_channel_state(deps.as_mut(), send_channel.to_string(),
         connection_id.to_string(),  counterparty_port_id.to_string(), 
         counterparty_channel_id.to_string()); 
    }


    #[test]
    fn test_receive_sg721_empty() {
        let send_channel = "channel-9";
        let mut deps = setup(&["channel-1", "channel-7", send_channel]);

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
            Attribute { key: "action".to_string(), value: "receive".to_string() },
             Attribute { key: "success".to_string(), value: "false".to_string() },
              Attribute { key: "error".to_string(), value: "NoSuchNft".to_string() }];
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

        let connection_id = "connection-2";
        let counterparty_port_id = "transfer-nft";
        let counterparty_channel_id = "channel-95";
        check_query_channel_state(deps.as_mut(), send_channel.to_string(),
         connection_id.to_string(),  counterparty_port_id.to_string(), 
         counterparty_channel_id.to_string()); 
    }

    #[test]
    fn test_send_sg721_success() {
        let send_channel = "channel-9";
        let mut deps = setup(&["channel-1", "channel-7", send_channel]);
        let contract_addr = "collection-addr";
        let token_ids = vec!["1", "2", "3"];
        let token_uris = vec![
            "https://metadata-url.com/my-metadata1",
            "https://metadata-url.com/my-metadata2",
            "https://metadata-url.com/my-metadata3",
        ];

        let res = send_sg721_success(deps.as_mut(), send_channel.to_string(), 
        contract_addr.to_string(), token_ids.clone(), token_uris.clone());

        assert_eq!(0, res.messages.len());
        
        let result_attributes = [
            Attribute { key: "action".to_string(), value: "acknowledge".to_string() },
            Attribute { key: "sender".to_string(), value: "local-sender".to_string() },
            Attribute { key: "receiver".to_string(), value: "remote-rcpt".to_string() }, 
            Attribute { key: "contract_addr".to_string(), value: "collection-addr".to_string() }, 
            Attribute { key: "success".to_string(), value: "true".to_string() }];
        assert_eq!(res.attributes, result_attributes);
        
        let connection_id = "connection-2";
        let counterparty_port_id = "transfer-nft";
        let counterparty_channel_id = "channel-95";
        check_query_channel_state(deps.as_mut(), send_channel.to_string(),
         connection_id.to_string(),  counterparty_port_id.to_string(), 
         counterparty_channel_id.to_string()); 
    }

    #[test]
    fn test_send_sg721_fail() {
        let send_channel = "channel-9";
        let mut deps = setup(&["channel-1", "channel-7", send_channel]);
        let contract_addr = "transfer-nft/abc/def";
        let token_ids = vec!["1", "2", "3"];
        let token_uris = vec![
            "https://metadata-url.com/my-metadata1",
            "https://metadata-url.com/my-metadata2",
            "https://metadata-url.com/my-metadata3",
        ];

        let ibc_packet =  mock_sent_packet(
            &send_channel,
            &contract_addr,
            token_ids.clone(),
            token_uris.clone(),
            "local-sender",
        );

        let mut contract_addr = ibc_packet.src.port_id.to_string(); 
        contract_addr += "/";
        contract_addr +=  &ibc_packet.src.channel_id.to_string(); 
        contract_addr += "/my-nft";

        let res = send_sg721_fail(deps.as_mut(), send_channel.to_string(), 
        contract_addr, token_ids.clone(), token_uris.clone());
        
        let reply_on = &res.messages[0].reply_on;
        let wasm_msg = &res.messages[0].msg;

        assert_eq!(reply_on, &ReplyOn::Error);
        let wasm_str = format!("{:?}", wasm_msg); 
        assert!(wasm_str.contains("contract_addr: \"my-nft\""));
        
        let res_attributes = [
            Attribute { key: "action".to_string(), value: "acknowledge".to_string() },
             Attribute { key: "sender".to_string(), value: "local-sender".to_string() }, 
             Attribute { key: "receiver".to_string(), value: "remote-rcpt".to_string() }, 
             Attribute { key: "contract_addr".to_string(), value: "my-nft".to_string() }, 
             Attribute { key: "success".to_string(), value: "false".to_string() },
            Attribute { key: "error".to_string(), value: "Packet Fail".to_string() }];
        assert_eq!(res.attributes, res_attributes);
        
    }

    // to add: 
    // println!("State class ids {}", _state.class_ids);
    // println!("State info is {}", _state.info);
    // assert_eq!(_state.balances, vec![Amount::cw20(987654321, cw721_addr)]);
    // assert_eq!(_state.total_sent, vec![Amount::cw20(987654321, cw721_addr)]);

    // // cannot receive more than we sent
    // let msg = IbcPacketReceiveMsg::new(recv_high_packet);
    // let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();
    // assert!(res.messages.is_empty());
    // let ack: Ics20Ack = from_binary(&res.acknowledgement).unwrap();
    // assert_eq!(ack, no_funds);

    // // we can receive less than we sent
    // let msg = IbcPacketReceiveMsg::new(recv_packet);
    // let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();
    // assert_eq!(1, res.messages.len());
    // assert_eq!(
    //     cw20_payment(876543210, cw721_addr, "local-rcpt"),
    //     res.messages[0]
    // );
    // let ack: Ics20Ack = from_binary(&res.acknowledgement).unwrap();
    // matches!(ack, Ics20Ack::Result(_));

    // // query channel state
    // let state = query_channel(deps.as_ref(), send_channel.to_string()).unwrap();
    // assert_eq!(state.balances, vec![Amount::cw20(111111111, cw721_addr)]);
    // assert_eq!(state.total_sent, vec![Amount::cw20(987654321, cw721_addr)]);
}