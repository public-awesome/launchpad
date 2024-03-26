#[cfg(test)]
mod tests {
    use crate::{
        contract::{execute, instantiate, query_config, query_has_member},
        msg::{ExecuteMsg, InstantiateMsg},
        tests::test_helpers::get_merkle_tree_simple,
    };
    use std::vec;

    use cosmwasm_std::{
        coin,
        testing::{mock_dependencies, mock_env, mock_info},
        BlockInfo, DepsMut, Env, Timestamp,
    };
    use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};

    const ADMIN: &str = "admin";
    const UNIT_AMOUNT: u128 = 100_000_000;

    const GENESIS_START_TIME: Timestamp = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    const END_TIME: Timestamp = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 1000);

    const MERKLE_ROOT: &str = "5ab281bca33c9819e0daa0708d20ff8a25e65de7d1f6659dbdeb1d2050652b80";
    const NON_HEX_MERKLE_ROOT: &str =
        "5zb281bca33c9819e0daa0708d20ff8a25e65de7d1f6659dbdeb1d2050652b80";
    const NON_32BYTES_MERKLE_ROOT: &str =
        "5ab281bca33c9819e0daa0708d20ff8a25e65de7d1f6659dbdeb1d2050652b80ab";

    fn setup_contract(deps: DepsMut, merkle_root: Option<String>) {
        let msg = InstantiateMsg {
            merkle_root: merkle_root.unwrap_or(MERKLE_ROOT.to_string()),
            merkle_tree_uri: None,
            per_address_limit: 1,
            start_time: GENESIS_START_TIME,
            end_time: END_TIME,
            mint_price: coin(UNIT_AMOUNT, NATIVE_DENOM),
            admins: vec![ADMIN.to_string()],
            admins_mutable: true,
        };
        let info = mock_info(ADMIN, &[]);
        let res = instantiate(deps, mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
        assert_eq!(5, res.attributes.len());
    }

    fn custom_mock_env() -> Env {
        Env {
            block: BlockInfo {
                height: 55_555,
                time: GENESIS_START_TIME.plus_nanos(100),
                chain_id: "stargaze-1".to_string(),
            },
            ..mock_env()
        }
    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut(), None);
    }

    #[test]
    fn improper_initializations() {
        let mut deps = mock_dependencies();
        let env = custom_mock_env();

        let invalid_msgs: Vec<InstantiateMsg> = vec![
            // invalid merkle root (non hex)
            InstantiateMsg {
                merkle_root: NON_HEX_MERKLE_ROOT.to_string(),
                merkle_tree_uri: None,
                per_address_limit: 1,
                start_time: GENESIS_START_TIME,
                end_time: END_TIME,
                mint_price: coin(1, NATIVE_DENOM),
                admins: vec![ADMIN.to_string()],
                admins_mutable: false,
            },
            // invalid merkle root (non 32 bytes)
            InstantiateMsg {
                merkle_root: NON_32BYTES_MERKLE_ROOT.to_string(),
                merkle_tree_uri: None,
                per_address_limit: 1,
                start_time: GENESIS_START_TIME,
                end_time: END_TIME,
                mint_price: coin(1, NATIVE_DENOM),
                admins: vec![ADMIN.to_string()],
                admins_mutable: false,
            },
            // invalid mint price denom
            InstantiateMsg {
                merkle_root: MERKLE_ROOT.to_string(),
                merkle_tree_uri: None,
                per_address_limit: 1,
                start_time: GENESIS_START_TIME,
                end_time: END_TIME,
                mint_price: coin(UNIT_AMOUNT, "not_ustars"),
                admins: vec![ADMIN.to_string()],
                admins_mutable: false,
            },
            // invalid admin address (MockApi only) (too short)
            InstantiateMsg {
                merkle_root: MERKLE_ROOT.to_string(),
                merkle_tree_uri: None,
                per_address_limit: 1,
                start_time: GENESIS_START_TIME,
                end_time: END_TIME,
                mint_price: coin(UNIT_AMOUNT, NATIVE_DENOM),
                admins: vec!["A".to_string()],
                admins_mutable: false,
            },
            // invalid start time (after end time)
            InstantiateMsg {
                merkle_root: MERKLE_ROOT.to_string(),
                merkle_tree_uri: None,
                per_address_limit: 1,
                start_time: END_TIME.plus_nanos(1u64),
                end_time: END_TIME,
                mint_price: coin(UNIT_AMOUNT, NATIVE_DENOM),
                admins: vec![ADMIN.to_string()],
                admins_mutable: false,
            },
            // invalid start time (before genesis mint start time)
            InstantiateMsg {
                merkle_root: MERKLE_ROOT.to_string(),
                merkle_tree_uri: None,
                per_address_limit: 1,
                start_time: GENESIS_START_TIME.minus_nanos(1u64),
                end_time: END_TIME,
                mint_price: coin(UNIT_AMOUNT, NATIVE_DENOM),
                admins: vec![ADMIN.to_string()],
                admins_mutable: false,
            },
            // invalid start time (before current block time)
            InstantiateMsg {
                merkle_root: MERKLE_ROOT.to_string(),
                merkle_tree_uri: None,
                per_address_limit: 1,
                start_time: env.block.time.minus_nanos(1u64),
                end_time: END_TIME,
                mint_price: coin(UNIT_AMOUNT, NATIVE_DENOM),
                admins: vec![ADMIN.to_string()],
                admins_mutable: false,
            },
        ];

        let info = mock_info(ADMIN, &[]);

        for msg in invalid_msgs {
            instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
        }
    }

    #[test]
    fn update_start_time() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut(), None);

        let msg = ExecuteMsg::UpdateStartTime(Timestamp::from_nanos(GENESIS_MINT_START_TIME - 100));
        let info = mock_info(ADMIN, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.attributes.len(), 3);
        let res = query_config(deps.as_ref(), mock_env()).unwrap();
        assert_eq!(res.start_time, GENESIS_START_TIME);
    }

    #[test]
    fn update_end_time() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut(), None);

        let msg = ExecuteMsg::UpdateEndTime(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 100));
        let info = mock_info(ADMIN, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.attributes.len(), 3);

        let msg = ExecuteMsg::UpdateEndTime(Timestamp::from_nanos(GENESIS_MINT_START_TIME - 100));
        let info = mock_info(ADMIN, &[]);
        execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    }

    #[test]
    fn query_membership() {
        let mut deps = mock_dependencies();

        let tree = get_merkle_tree_simple(None);
        let root = tree.root_hex();

        setup_contract(deps.as_mut(), root.clone());

        let proof = tree.proof(&[0]);
        let hash_strings = proof.proof_hashes_hex();

        let user = mock_info("stars1ye63jpm474yfrq02nyplrspyw75y82tptsls9t", &[]);
        let res = query_has_member(deps.as_ref(), user.sender.to_string(), hash_strings).unwrap();
        assert!(res.has_member);

        let user = mock_info("stars130dxx3nr2ste4fwsum57k3en60wqd76m9pvpsy", &[]);
        let proof = vec![
            "ea930af5025204fc0dda1b69b567b6b41766107d65d46a1acd9725af65604531".to_string(),
            "4c0801ba42388ec349cfeae552dfa64271008f37b10c2601e32c0cf2729c0278".to_string(),
        ];
        let res = query_has_member(deps.as_ref(), user.sender.to_string(), proof).unwrap();
        assert!(res.has_member);

        let user = mock_info("stars16epdu6c7h8apxrnuu06yzfxflrede0mtu4qqz4", &[]);
        let proof = vec![
            "28fc41471ab92238e98664e99671e906cb29c048dd0343f3acf5295e424270e1".to_string(),
            "8e9abbdd48390cd7ed2d6f6934b713f7839801716ad8b5ae674c1d682db6de34".to_string(),
        ];
        let res = query_has_member(deps.as_ref(), user.sender.to_string(), proof).unwrap();
        assert!(res.has_member);

        // mismatched proof
        let user = mock_info("stars1ye63jpm474yfrq02nyplrspyw75y82tptsls9t", &[]);
        let proof = vec![
            "ea930af5025204fc0dda1b69b567b6b41766107d65d46a1acd9725af65604531".to_string(),
            "28fc41471ab92238e98664e99671e906cb29c048dd0343f3acf5295e424270e1".to_string(),
        ];
        let res = query_has_member(deps.as_ref(), user.sender.to_string(), proof).unwrap();
        assert!(!res.has_member);

        // invalid proof
        let user = mock_info("stars1ye63jpm474yfrq02nyplrspyw75y82tptsls9t", &[]);
        let proof = vec!["x".to_string(), "x".to_string()];
        let _ = query_has_member(deps.as_ref(), user.sender.to_string(), proof).unwrap_err();
    }
}
