#[cfg(test)]
mod tests {
    use cosmwasm_std::Addr;
    use cw_multi_test::Executor;
    use sg_multi_test::StargazeApp;
    use whitelist_immutable_flex::msg::*;
    use whitelist_immutable_flex::{helpers::WhitelistImmutableFlexContract, state::Config};

    use crate::common_setup::contract_boxes::{contract_whitelist_immutable_flex, custom_mock_app};

    const CREATOR: &str = "creator";

    fn get_init_address_list_1() -> Vec<Member> {
        vec![
            Member {
                address: "addr0001".to_string(),
                mint_count: 1,
            },
            Member {
                address: "addr0002".to_string(),
                mint_count: 2,
            },
            Member {
                address: "addr0003".to_string(),
                mint_count: 3,
            },
            Member {
                address: "addr0004".to_string(),
                mint_count: 4,
            },
            Member {
                address: "addr0005".to_string(),
                mint_count: 5,
            },
        ]
    }

    fn get_init_address_list_2() -> Vec<Member> {
        vec![
            Member {
                address: "tester".to_string(),
                mint_count: 1,
            },
            Member {
                address: "user".to_string(),
                mint_count: 2,
            },
            Member {
                address: "random".to_string(),
                mint_count: 3,
            },
            Member {
                address: "human".to_string(),
                mint_count: 4,
            },
            Member {
                address: "bot".to_string(),
                mint_count: 5,
            },
        ]
    }

    fn get_init_address_single_list() -> Vec<Member> {
        vec![Member {
            address: "onlyone".to_string(),
            mint_count: 1,
        }]
    }

    pub fn instantiate_with_addresses(app: &mut StargazeApp, members: Vec<Member>) -> Addr {
        let msg = InstantiateMsg {
            members,
            mint_discount_bps: None,
        };
        let wl_id = app.store_code(contract_whitelist_immutable_flex());
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

    pub fn query_address_count(app: &mut StargazeApp, addrs: Vec<Member>, wl_addr: Addr) {
        let count: u64 = app
            .wrap()
            .query_wasm_smart(wl_addr, &QueryMsg::AddressCount {})
            .unwrap();
        assert_eq!(count, addrs.len() as u64);
    }

    pub fn query_admin(app: &mut StargazeApp, wl_addr: Addr) {
        let admin: String = app
            .wrap()
            .query_wasm_smart(wl_addr, &QueryMsg::Admin {})
            .unwrap();
        assert_eq!(admin, CREATOR.to_string());
    }

    pub fn query_includes_address(app: &mut StargazeApp, wl_addr: Addr, addr_to_check: String) {
        let includes: bool = app
            .wrap()
            .query_wasm_smart(
                wl_addr,
                &QueryMsg::HasMember {
                    address: addr_to_check,
                },
            )
            .unwrap();
        assert!(includes);
    }

    pub fn query_member(
        app: &mut StargazeApp,
        wl_addr: Addr,
        member_address: String,
        expected_mint_count: u32,
    ) {
        let member: Member = app
            .wrap()
            .query_wasm_smart(
                wl_addr,
                &QueryMsg::Member {
                    address: member_address.clone(),
                },
            )
            .unwrap();
        assert_eq!(member.address, member_address);
        assert_eq!(member.mint_count, expected_mint_count);
    }
    #[test]
    pub fn test_instantiate() {
        let mut app = custom_mock_app();
        let addrs = get_init_address_list_2();
        let wl_addr = instantiate_with_addresses(&mut app, addrs.clone());
        let addr_to_check = addrs[1].clone().address;
        query_admin(&mut app, wl_addr.clone());
        query_address_count(&mut app, addrs, wl_addr.clone());
        query_includes_address(&mut app, wl_addr.clone(), addr_to_check.clone());
        query_member(&mut app, wl_addr.clone(), addr_to_check, 2);
    }

    #[test]
    pub fn test_instantiate_single_list() {
        let mut app = custom_mock_app();
        let addrs = get_init_address_single_list();
        let wl_addr = instantiate_with_addresses(&mut app, addrs.clone());
        let addr_to_check = addrs[0].clone().address;
        query_admin(&mut app, wl_addr.clone());
        query_address_count(&mut app, addrs, wl_addr.clone());
        query_includes_address(&mut app, wl_addr.clone(), addr_to_check.clone());
        query_member(&mut app, wl_addr.clone(), addr_to_check, 1);
    }

    #[test]
    pub fn test_instantiate_empty_list_error() {
        let mut app = custom_mock_app();
        let members = vec![];

        let msg = InstantiateMsg {
            members,
            mint_discount_bps: None,
        };
        let wl_id = app.store_code(contract_whitelist_immutable_flex());
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
    pub fn test_helper_query_address_count() {
        let mut app = custom_mock_app();
        let addrs = get_init_address_single_list();
        let wl_addr = instantiate_with_addresses(&mut app, addrs);

        let res: u64 = WhitelistImmutableFlexContract(wl_addr)
            .address_count(&app.wrap())
            .unwrap();
        assert_eq!(res, 1);
    }

    #[test]
    pub fn test_helper_query_config() {
        let mut app = custom_mock_app();
        let addrs = get_init_address_single_list();
        let wl_addr = instantiate_with_addresses(&mut app, addrs);

        let res: Config = WhitelistImmutableFlexContract(wl_addr)
            .config(&app.wrap())
            .unwrap();
        let expected_config = Config {
            admin: Addr::unchecked("creator"),
            mint_discount_bps: None,
        };
        assert_eq!(res, expected_config);
    }

    #[test]
    pub fn test_helper_query_includes() {
        let mut app = custom_mock_app();
        let addrs = get_init_address_list_1();
        let wl_addr = instantiate_with_addresses(&mut app, addrs);

        let res: bool = WhitelistImmutableFlexContract(wl_addr.clone())
            .includes(&app.wrap(), "addr0003".to_string())
            .unwrap();
        assert!(res);

        let res: bool = WhitelistImmutableFlexContract(wl_addr)
            .includes(&app.wrap(), "nonsense".to_string())
            .unwrap();
        assert!(!res);
    }
}
