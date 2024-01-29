#[cfg(test)]
mod tests {
    use cosmwasm_std::{coin, Addr, Timestamp};
    use cw_multi_test::Executor;
    use rs_merkle::MerkleTree;

    use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};

    use whitelist_mtree::{
        msg::*,
        tests::{hasher::SortingSha256Hasher, test_helpers::hash_and_build_tree},
    };

    use crate::common_setup::contract_boxes::{
        contract_whitelist_merkletree, custom_mock_app, App,
    };

    type Tree = MerkleTree<SortingSha256Hasher>;

    const CREATOR: &str = "creator";
    const START_TIME: Timestamp = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    const END_TIME: Timestamp = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 1000);

    fn get_init_address_list_1() -> Vec<String> {
        vec![
            "addr0001".to_string(),
            "addr0002".to_string(),
            "addr0003".to_string(),
            "addr0004".to_string(),
            "addr0005".to_string(),
        ]
    }

    fn get_init_address_list_2() -> Vec<String> {
        vec![
            "tester".to_string(),
            "user".to_string(),
            "rando".to_string(),
            "human".to_string(),
            "bot".to_string(),
        ]
    }

    fn get_init_address_single_list() -> Vec<String> {
        vec!["onlyone".to_string()]
    }

    pub fn instantiate_with_root(
        app: &mut App,
        per_address_limit: u32,
        merkle_root: String,
    ) -> Addr {
        let msg = InstantiateMsg {
            admins: vec![],
            admins_mutable: false,
            start_time: START_TIME,
            end_time: END_TIME,
            mint_price: coin(1000000u128, NATIVE_DENOM),
            per_address_limit,
            merkle_root,
            merkle_tree_uri: None,
        };
        let wl_id = app.store_code(contract_whitelist_merkletree());
        app.instantiate_contract(
            wl_id,
            Addr::unchecked(CREATOR),
            &msg,
            &[],
            "wl-contract-mtree".to_string(),
            None,
        )
        .unwrap()
    }

    pub fn query_admin_list(app: &mut App, wl_addr: Addr) {
        let res: AdminListResponse = app
            .wrap()
            .query_wasm_smart(wl_addr, &QueryMsg::AdminList {})
            .unwrap();
        assert_eq!(res.admins.len(), 0);
        assert!(!res.mutable)
    }

    pub fn query_includes_address(
        app: &mut App,
        wl_addr: Addr,
        addr_to_check: String,
        proof_hashes: Vec<String>,
    ) {
        let res: HasMemberResponse = app
            .wrap()
            .query_wasm_smart(
                wl_addr,
                &QueryMsg::HasMember {
                    member: addr_to_check.to_string(),
                    proof_hashes,
                },
            )
            .unwrap();
        assert!(res.has_member);
    }

    pub fn query_per_address_limit(app: &mut App, wl_addr: Addr, per_address_limit: u32) {
        let config: ConfigResponse = app
            .wrap()
            .query_wasm_smart(wl_addr, &QueryMsg::Config {})
            .unwrap();
        assert_eq!(config.per_address_limit, per_address_limit);
    }

    #[test]
    pub fn test_instantiate_with_one_mint() {
        let mut app = custom_mock_app();
        let addrs = get_init_address_list_1();
        let per_address_limit = 1;
        let tree: Tree = hash_and_build_tree(&addrs);
        let wl_addr = instantiate_with_root(&mut app, per_address_limit, tree.root_hex().unwrap());

        let addr_to_check = addrs[0].clone();
        let proof = tree.proof(&[0]);
        let proof_hashes = proof.proof_hashes_hex();

        query_includes_address(&mut app, wl_addr.clone(), addr_to_check, proof_hashes);
        // execute_query_checks(&mut app, wl_addr, addrs, per_address_limit, addr_to_check);
        query_admin_list(&mut app, wl_addr.clone());
        query_per_address_limit(&mut app, wl_addr, per_address_limit)
    }
    #[test]
    pub fn test_instantiate_with_multiple_mints() {
        let mut app = custom_mock_app();
        let addrs = get_init_address_list_2();
        let per_address_limit = 99;

        let tree: Tree = hash_and_build_tree(&addrs);
        let wl_addr = instantiate_with_root(&mut app, per_address_limit, tree.root_hex().unwrap());
        let addr_to_check = addrs[1].clone();

        let proof = tree.proof(&[1]);
        let proof_hashes = proof.proof_hashes_hex();

        query_admin_list(&mut app, wl_addr.clone());
        query_includes_address(&mut app, wl_addr.clone(), addr_to_check, proof_hashes);
        query_per_address_limit(&mut app, wl_addr, per_address_limit)
    }

    #[test]
    pub fn test_instantiate_single_list() {
        let mut app = custom_mock_app();
        let addrs = get_init_address_single_list();
        let per_address_limit = 5;

        let tree: Tree = hash_and_build_tree(&addrs);
        let wl_addr = instantiate_with_root(&mut app, per_address_limit, tree.root_hex().unwrap());

        let addr_to_check = addrs[0].clone();
        query_admin_list(&mut app, wl_addr.clone());

        let proof = tree.proof(&[0]);
        let proof_hashes = proof.proof_hashes_hex();

        query_includes_address(&mut app, wl_addr.clone(), addr_to_check, proof_hashes);
        query_per_address_limit(&mut app, wl_addr, per_address_limit)
    }
}
