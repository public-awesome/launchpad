#[cfg(test)]
mod tests {
    use cosmwasm_std::Addr;
    use cw_multi_test::Executor;
    use whitelist_immutable::msg::*;
    use whitelist_immutable::{helpers::WhitelistImmutableContract, state::Config};

    use crate::common_setup::contract_boxes::{contract_whitelist_immutable, custom_mock_app, App};

    const CREATOR: &str = "creator";

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

    pub fn instantiate_with_addresses(
        app: &mut App,
        addrs: Vec<String>,
        per_address_limit: u32,
    ) -> Addr {
        let msg = InstantiateMsg {
            per_address_limit,
            addresses: addrs,
            mint_discount_bps: None,
        };
        let wl_id = app.store_code(contract_whitelist_immutable());
        app.instantiate_contract(
            wl_id,
            Addr::unchecked(CREATOR),
            &msg,
            &[],
            "wl-contract".to_string(),
            None,
        )
        .unwrap()
    }

    pub fn query_address_count(app: &mut App, addrs: Vec<String>, wl_addr: Addr) {
        let count: u64 = app
            .wrap()
            .query_wasm_smart(wl_addr, &QueryMsg::AddressCount {})
            .unwrap();
        assert_eq!(count, addrs.len() as u64);
    }

    pub fn query_admin(app: &mut App, wl_addr: Addr) {
        let admin: String = app
            .wrap()
            .query_wasm_smart(wl_addr, &QueryMsg::Admin {})
            .unwrap();
        assert_eq!(admin, CREATOR.to_string());
    }

    pub fn query_includes_address(app: &mut App, wl_addr: Addr, addr_to_check: String) {
        let includes: bool = app
            .wrap()
            .query_wasm_smart(
                wl_addr,
                &QueryMsg::IncludesAddress {
                    address: addr_to_check,
                },
            )
            .unwrap();
        assert!(includes);
    }

    pub fn query_per_address_limit(app: &mut App, wl_addr: Addr, per_address_limit: u32) {
        let limit: u32 = app
            .wrap()
            .query_wasm_smart(wl_addr, &QueryMsg::PerAddressLimit {})
            .unwrap();
        assert_eq!(limit, per_address_limit);
    }

    #[test]
    pub fn test_instantiate_with_one_mint() {
        let mut app = custom_mock_app();
        let addrs = get_init_address_list_1();
        let per_address_limit = 1;
        let wl_addr = instantiate_with_addresses(&mut app, addrs.clone(), per_address_limit);
        let addr_to_check = addrs[0].clone();
        // execute_query_checks(&mut app, wl_addr, addrs, per_address_limit, addr_to_check);
        query_admin(&mut app, wl_addr.clone());
        query_address_count(&mut app, addrs, wl_addr.clone());
        query_includes_address(&mut app, wl_addr.clone(), addr_to_check);
        query_per_address_limit(&mut app, wl_addr, per_address_limit)
    }
    #[test]
    pub fn test_instantiate_with_multiple_mints() {
        let mut app = custom_mock_app();
        let addrs = get_init_address_list_2();
        let per_address_limit = 99;
        let wl_addr = instantiate_with_addresses(&mut app, addrs.clone(), per_address_limit);
        let addr_to_check = addrs[1].clone();
        query_admin(&mut app, wl_addr.clone());
        query_address_count(&mut app, addrs, wl_addr.clone());
        query_includes_address(&mut app, wl_addr.clone(), addr_to_check);
        query_per_address_limit(&mut app, wl_addr, per_address_limit)
    }

    #[test]
    pub fn test_instantiate_single_list() {
        let mut app = custom_mock_app();
        let addrs = get_init_address_single_list();
        let per_address_limit = 5;
        let wl_addr = instantiate_with_addresses(&mut app, addrs.clone(), per_address_limit);
        let addr_to_check = addrs[0].clone();
        query_admin(&mut app, wl_addr.clone());
        query_address_count(&mut app, addrs, wl_addr.clone());
        query_includes_address(&mut app, wl_addr.clone(), addr_to_check);
        query_per_address_limit(&mut app, wl_addr, per_address_limit)
    }

    #[test]
    pub fn test_instantiate_empty_list_error() {
        let mut app = custom_mock_app();
        let addrs = vec![];
        let per_address_limit = 5;

        let msg = InstantiateMsg {
            per_address_limit,
            addresses: addrs,
            mint_discount_bps: None,
        };
        let wl_id = app.store_code(contract_whitelist_immutable());
        let res = app
            .instantiate_contract(
                wl_id,
                Addr::unchecked(CREATOR),
                &msg,
                &[],
                "wl-contract".to_string(),
                None,
            )
            .unwrap_err();
        let expected_error = "Empty whitelist, must provide at least one address";
        assert_eq!(res.root_cause().to_string(), expected_error)
    }

    #[test]
    pub fn test_helper_query_address_limit() {
        let mut app = custom_mock_app();
        let addrs = get_init_address_single_list();
        let per_address_limit = 30;
        let wl_addr = instantiate_with_addresses(&mut app, addrs, per_address_limit);

        let res: u32 = WhitelistImmutableContract(wl_addr)
            .per_address_limit(&app.wrap())
            .unwrap();
        assert_eq!(res, 30);
    }

    #[test]
    pub fn test_helper_query_address_count() {
        let mut app = custom_mock_app();
        let addrs = get_init_address_single_list();
        let per_address_limit = 10;
        let wl_addr = instantiate_with_addresses(&mut app, addrs, per_address_limit);

        let res: u64 = WhitelistImmutableContract(wl_addr)
            .address_count(&app.wrap())
            .unwrap();
        assert_eq!(res, 1);
    }

    #[test]
    pub fn test_helper_query_config() {
        let mut app = custom_mock_app();
        let addrs = get_init_address_single_list();
        let per_address_limit = 10;
        let wl_addr = instantiate_with_addresses(&mut app, addrs, per_address_limit);

        let res: Config = WhitelistImmutableContract(wl_addr)
            .config(&app.wrap())
            .unwrap();
        let expected_config = Config {
            admin: Addr::unchecked("creator"),
            per_address_limit: 10,
            mint_discount_bps: None,
        };
        assert_eq!(res, expected_config);
    }

    #[test]
    pub fn test_helper_query_includes() {
        let mut app = custom_mock_app();
        let addrs = get_init_address_list_1();
        let per_address_limit = 10;
        let wl_addr = instantiate_with_addresses(&mut app, addrs, per_address_limit);

        let res: bool = WhitelistImmutableContract(wl_addr.clone())
            .includes(&app.wrap(), "addr0003".to_string())
            .unwrap();
        assert!(res);

        let res: bool = WhitelistImmutableContract(wl_addr)
            .includes(&app.wrap(), "nonsense".to_string())
            .unwrap();
        assert!(!res);
    }
}
