use crate::integration_tests::tests::instantiate_with_addresses;

#[cfg(test)]
mod tests {
    use crate::msg::*;

    use cosmwasm_std::Addr;
    use sg_std::StargazeMsgWrapper;

    use cw_multi_test::{Contract, ContractWrapper, Executor};

    use sg_multi_test::StargazeApp;

    const CREATOR: &str = "creator";
    const OTHER_ADMIN: &str = "other_admin";
    const PER_ADDRESS_LIMIT: u32 = 10;

    fn custom_mock_app() -> StargazeApp {
        StargazeApp::default()
    }

    pub fn wl_contract() -> Box<dyn Contract<StargazeMsgWrapper>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    fn get_init_address_list() -> Vec<String> {
        vec![
            "addr0001".to_string(),
            "addr0002".to_string(),
            "addr0003".to_string(),
            "addr0004".to_string(),
            "addr0005".to_string(),
        ]
    }

    pub fn instantiate_with_addresses(app: &mut StargazeApp) -> Addr {
        let addrs = get_init_address_list();    
        let msg = InstantiateMsg {
            per_address_limit: PER_ADDRESS_LIMIT,
            addresses: addrs.clone(),
            mint_discount_bps: None,
        };
        let wl_id = app.store_code(wl_contract());
        let wl_addr = app
            .instantiate_contract(
                wl_id,
                Addr::unchecked(CREATOR),
                &msg,
                &[],
                "wl-contract".to_string(),
                None,
            )
            .unwrap();
            wl_addr
        }
    
    #[test]
    pub fn test_instantiate_with_addresses() {
        let mut app = custom_mock_app();
        instantiate_with_addresses(&mut app);
    }

    #[test]
    pub fn test_wl_contract_initial_state_query() {
        let mut app = custom_mock_app();
        let wl_addr = instantiate_with_addresses(&mut app);
        let addrs = get_init_address_list();

        let admin: String = app
            .wrap()
            .query_wasm_smart(&wl_addr, &QueryMsg::Admin {})
            .unwrap();
        assert_eq!(admin, CREATOR.to_string());

        let count: u64 = app
            .wrap()
            .query_wasm_smart(&wl_addr, &QueryMsg::AddressCount {})
            .unwrap();
        assert_eq!(count, addrs.len() as u64);

        let includes: bool = app
            .wrap()
            .query_wasm_smart(
                &wl_addr,
                &QueryMsg::IncludesAddress {
                    address: addrs[0].clone(),
                },
            )
            .unwrap();
        assert!(includes);

        let count: u32 = app
            .wrap()
            .query_wasm_smart(
                &wl_addr,
                &QueryMsg::MintCount {
                    address: addrs[0].clone(),
                },
            )
            .unwrap();
        assert_eq!(count, 0);

        let limit: u32 = app
            .wrap()
            .query_wasm_smart(&wl_addr, &QueryMsg::PerAddressLimit {})
            .unwrap();
        assert_eq!(limit, 10);
    }

    #[test]
    pub fn test_process_address() {
        let mut app = custom_mock_app();
        let wl_addr = instantiate_with_addresses(&mut app);
        let addrs = get_init_address_list();

        let msg = ExecuteMsg::ProcessAddress {
            address: addrs[0].clone(),
        };
        let res = app.execute_contract(Addr::unchecked(CREATOR), wl_addr.clone(), &msg, &[]);
        assert!(res.is_ok());
        let res: u32 = app
            .wrap()
            .query_wasm_smart(
                &wl_addr,
                &QueryMsg::MintCount {
                    address: addrs[0].clone(),
                },
            )
            .unwrap();
        assert_eq!(res, 1);
    }

    #[test]
    fn test_update_admin() {
        let mut app = custom_mock_app();
        let wl_addr = instantiate_with_addresses(&mut app);

        let msg = ExecuteMsg::UpdateAdmin {
            new_admin: OTHER_ADMIN.to_string(),
        };
        let res = app.execute_contract(Addr::unchecked(CREATOR), wl_addr.clone(), &msg, &[]);
        assert!(res.is_ok());
        let res: String = app
            .wrap()
            .query_wasm_smart(&wl_addr, &QueryMsg::Admin {})
            .unwrap();
        assert_eq!(res, OTHER_ADMIN.to_string());
    }
    #[test]
    pub fn test_add_existing_addresses_fail() {
        let mut app = custom_mock_app();
        let wl_addr = instantiate_with_addresses(&mut app);

        // add addresses
        let msg = ExecuteMsg::AddAddresses {
            addresses: vec![
                "addr0001".to_string(),
                "addr0002".to_string(),
                "addr0003".to_string(),
                "addr0004".to_string(),
                "addr0006".to_string(),
            ],
        };
        let res = app.execute_contract(Addr::unchecked(OTHER_ADMIN), wl_addr.clone(), &msg, &[]);
        assert!(res.is_err());
        let res: bool = app
            .wrap()
            .query_wasm_smart(
                &wl_addr,
                &QueryMsg::IncludesAddress {
                    address: "addr0006".to_string(),
                },
            )
            .unwrap();
        assert!(!res);
        let res: u64 = app
            .wrap()
            .query_wasm_smart(&wl_addr, &QueryMsg::AddressCount {})
            .unwrap();
        assert_eq!(res, 5);

    }

    pub fn add_addresses(addresses: Vec<String> ) {
        let mut app = custom_mock_app();
        let wl_addr = instantiate_with_addresses(&mut app);

        let msg = ExecuteMsg::AddAddresses {
            addresses: addresses,
        };
        let res = app.execute_contract(Addr::unchecked(CREATOR), wl_addr.clone(), &msg, &[]);
        assert!(res.is_ok());
    }

    #[test]
    pub fn add_addresses_success() {
        let addresses = vec!["addr0007".to_string(), "addr0006".to_string()];
        add_addresses(addresses);
    }

    #[test]
    pub fn remove_addresses_fail(){
        let mut app = custom_mock_app();
        let wl_addr = instantiate_with_addresses(&mut app);
        let addresses = vec!["addr0007".to_string(), "addr0006".to_string()];
        add_addresses(addresses);

        let res: bool = app
            .wrap()
            .query_wasm_smart(
                &wl_addr,
                &QueryMsg::IncludesAddress {
                    address: "addr0006".to_string(),
                },
            )
            .unwrap();
        println!("res is {:?}", res);
        assert!(res);
        let res: u64 = app
            .wrap()
            .query_wasm_smart(&wl_addr, &QueryMsg::AddressCount {})
            .unwrap();
        assert_eq!(res, 7);

        // remove addresses
        let msg = ExecuteMsg::RemoveAddresses {
            addresses: vec![
                "addr0000".to_string(),
                "addr0001".to_string(),
                "addr0002".to_string(),
                "addr0003".to_string(),
                "addr0004".to_string(),
                "addr0006".to_string(),
            ],
        };
        let res = app.execute_contract(Addr::unchecked(CREATOR), wl_addr.clone(), &msg, &[]);
        println!("res is {:?}", res);
        assert!(res.is_err());
    }

//         let msg = ExecuteMsg::RemoveAddresses {
//             addresses: vec![
//                 "addr0001".to_string(),
//                 "addr0002".to_string(),
//                 "addr0003".to_string(),
//                 "addr0004".to_string(),
//                 "addr0006".to_string(),
//             ],
//         };
//         let res = app.execute_contract(Addr::unchecked(OTHER_ADMIN), wl_addr.clone(), &msg, &[]);
//         assert!(res.is_ok());
//         let res: bool = app
//             .wrap()
//             .query_wasm_smart(
//                 &wl_addr,
//                 &QueryMsg::IncludesAddress {
//                     address: "addr0006".to_string(),
//                 },
//             )
//             .unwrap();
//         assert!(!res);
//         let res: u64 = app
//             .wrap()
//             .query_wasm_smart(&wl_addr, &QueryMsg::AddressCount {})
//             .unwrap();
//         assert_eq!(res, 2);

//         // per address limit
//         let new_per_address_limit = 1;
//         let msg = ExecuteMsg::UpdatePerAddressLimit {
//             limit: new_per_address_limit,
//         };
//         let res = app.execute_contract(Addr::unchecked(OTHER_ADMIN), wl_addr.clone(), &msg, &[]);
//         assert!(res.is_ok());
//         let res: u32 = app
//             .wrap()
//             .query_wasm_smart(&wl_addr, &QueryMsg::PerAddressLimit {})
//             .unwrap();
//         assert_eq!(res, 1);

//         // surpass limit
//         let res: bool = app
//             .wrap()
//             .query_wasm_smart(
//                 &wl_addr,
//                 &QueryMsg::IsProcessable {
//                     address: "addr0007".to_string(),
//                 },
//             )
//             .unwrap();
//         assert!(res);
//         let msg = ExecuteMsg::ProcessAddress {
//             address: "addr0007".to_string(),
//         };
//         let res = app.execute_contract(
//             Addr::unchecked(OTHER_ADMIN.clone()),
//             wl_addr.clone(),
//             &msg,
//             &[],
//         );

//         assert!(res.is_ok());
//         let res: bool = app
//             .wrap()
//             .query_wasm_smart(
//                 &wl_addr,
//                 &QueryMsg::IsProcessable {
//                     address: "addr0007".to_string(),
//                 },
//             )
//             .unwrap();
//         assert!(!res);
//         let msg = ExecuteMsg::ProcessAddress {
//             address: "addr0007".to_string(),
//         };
//         let res = app.execute_contract(
//             Addr::unchecked(OTHER_ADMIN.clone()),
//             wl_addr.clone(),
//             &msg,
//             &[],
//         );
//         assert!(res.is_err());

//         // purge
//         let msg = ExecuteMsg::Purge {};
//         let res = app.execute_contract(Addr::unchecked(OTHER_ADMIN), wl_addr.clone(), &msg, &[]);
//         assert!(res.is_ok());
//         let res: u32 = app
//             .wrap()
//             .query_wasm_smart(&wl_addr, &QueryMsg::AddressCount {})
//             .unwrap();
//         assert_eq!(res, 0);
//         // does not include addr0007
//         let res: bool = app
//             .wrap()
//             .query_wasm_smart(
//                 &wl_addr,
//                 &QueryMsg::IncludesAddress {
//                     address: "addr0007".to_string(),
//                 },
//             )
//             .unwrap();
//         assert!(!res);

//         // query config
//         let res: ConfigResponse = app
//             .wrap()
//             .query_wasm_smart(&wl_addr, &QueryMsg::Config {})
//             .unwrap();
//         assert_eq!(res.config.admin, Addr::unchecked(OTHER_ADMIN).to_string());
//         assert_eq!(res.config.per_address_limit, new_per_address_limit);
//     }
}
