#[cfg(test)]
mod tests {
    use crate::common_setup::{contract_boxes_empty::contract_group, helpers::member};
    use crate::common_setup::{
        contract_boxes_empty::contract_splits, helpers::mock_app_builder_init_funds,
    };
    use cosmwasm_std::{to_json_binary, Addr, Coin};
    use cw2::{query_contract_info, ContractVersion};
    use cw4::{Cw4ExecuteMsg, Member, MemberListResponse};
    use cw4_group::msg::ExecuteMsg as Cw4GroupExecuteMsg;
    use cw_multi_test::{next_block, App, Executor as TestExecutor, IntoAddr};
    use sg_controllers::ContractInstantiateMsg;
    use sg_splits::contract::MAX_GROUP_SIZE;
    use sg_splits::msg::Group;
    use sg_splits::{
        msg::{InstantiateMsg, QueryMsg},
        ContractError,
    };
    use std::collections::HashMap;

    fn owner() -> Addr {
        "admin0001".into_addr()
    }
    fn member1() -> Addr {
        "member0001".into_addr()
    }
    fn member2() -> Addr {
        "member0002".into_addr()
    }
    fn member3() -> Addr {
        "member0003".into_addr()
    }

    // uploads code and returns address of group contract
    fn instantiate_group(app: &mut App, members: Vec<Member>) -> Addr {
        let group_id = app.store_code(contract_group());
        let msg = cw4_group::msg::InstantiateMsg {
            admin: Some(owner().to_string()),
            members,
        };
        app.instantiate_contract(group_id, owner(), &msg, &[], "group", None)
            .unwrap()
    }

    #[track_caller]
    fn instantiate_splits_with_group(app: &mut App, group_addr: Addr) -> Addr {
        let flex_id = app.store_code(contract_splits());
        let msg = sg_splits::msg::InstantiateMsg {
            group: Group::Cw4Address(group_addr.to_string()),
            admin: None,
        };
        app.instantiate_contract(flex_id, owner(), &msg, &[], "splits", None)
            .unwrap()
    }

    #[track_caller]
    fn instantiate_splits(app: &mut App) -> Addr {
        let flex_id = app.store_code(contract_splits());
        let group_msg = cw4_group::msg::InstantiateMsg {
            admin: Some(owner().to_string()),
            members: vec![
                member(owner().to_string(), 50),
                member(member1().to_string(), 25),
                member(member2().to_string(), 20),
                member(member3().to_string(), 5),
            ],
        };

        let msg = sg_splits::msg::InstantiateMsg {
            group: Group::Cw4Instantiate(ContractInstantiateMsg {
                code_id: app.store_code(contract_group()),
                msg: to_json_binary(&group_msg).unwrap(),
                admin: None,
                label: "cw4-group".to_string(),
            }),
            admin: Some(owner().to_string()),
        };
        app.instantiate_contract(flex_id, owner(), &msg, &[], "splits", None)
            .unwrap()
    }

    #[track_caller]
    fn instantiate_splits_with_overflow_group(app: &mut App) -> Addr {
        let flex_id = app.store_code(contract_splits());

        let members: Vec<Member> = (1..=MAX_GROUP_SIZE + 1)
            .map(|i| member(format!("member{i:04}").into_addr().to_string(), 1))
            .collect();

        let group_msg = cw4_group::msg::InstantiateMsg {
            admin: Some(owner().to_string()),
            members,
        };

        let msg = sg_splits::msg::InstantiateMsg {
            group: Group::Cw4Instantiate(ContractInstantiateMsg {
                code_id: app.store_code(contract_group()),
                msg: to_json_binary(&group_msg).unwrap(),
                admin: None,
                label: "cw4-group".to_string(),
            }),
            admin: Some(owner().to_string()),
        };
        app.instantiate_contract(flex_id, owner(), &msg, &[], "splits", None)
            .unwrap()
    }

    #[track_caller]
    fn setup_test_case(
        app: &mut App,
        init_funds: Vec<Coin>,
        multisig_as_group_admin: bool,
    ) -> (Addr, Addr) {
        // 1. Instantiate group contract with members (and owner as admin)
        let members = vec![
            member(owner().to_string(), 50),
            member(member1().to_string(), 25),
            member(member2().to_string(), 20),
            member(member3().to_string(), 5),
        ];
        let group_addr = instantiate_group(app, members);
        app.update_block(next_block);

        // 2. Set up Splits backed by this group
        let splits_addr = instantiate_splits_with_group(app, group_addr.clone());
        app.update_block(next_block);

        // 3. (Optional) Set the multisig as the group owner
        if multisig_as_group_admin {
            let update_admin = Cw4ExecuteMsg::UpdateAdmin {
                admin: Some(splits_addr.to_string()),
            };
            app.execute_contract(owner(), group_addr.clone(), &update_admin, &[])
                .unwrap();
            app.update_block(next_block);
        }

        // Bonus: set some funds on the splits contract for future proposals
        if !init_funds.is_empty() {
            app.send_tokens(owner(), splits_addr.clone(), &init_funds)
                .unwrap();
        }
        (splits_addr, group_addr)
    }

    #[track_caller]
    fn setup_test_case_with_internal_group(app: &mut App, init_funds: Vec<Coin>) -> (Addr, Addr) {
        // Set up Splits with internal group
        let splits_addr = instantiate_splits(app);
        app.update_block(next_block);

        // Bonus: set some funds on the splits contract for future proposals
        if !init_funds.is_empty() {
            app.send_tokens(owner(), splits_addr.clone(), &init_funds)
                .unwrap();
        }

        let group_addr: Addr = app
            .wrap()
            .query_wasm_smart(&splits_addr, &QueryMsg::Group {})
            .unwrap();

        (splits_addr, group_addr)
    }

    #[track_caller]
    fn setup_test_case_with_overflow_group(app: &mut App, init_funds: Vec<Coin>) -> (Addr, Addr) {
        // Set up Splits with internal group
        let splits_addr = instantiate_splits_with_overflow_group(app);
        app.update_block(next_block);

        // Bonus: set some funds on the splits contract for future proposals
        if !init_funds.is_empty() {
            app.send_tokens(owner(), splits_addr.clone(), &init_funds)
                .unwrap();
        }

        let group_addr: Addr = app
            .wrap()
            .query_wasm_smart(&splits_addr, &QueryMsg::Group {})
            .unwrap();

        (splits_addr, group_addr)
    }

    #[test]
    fn test_instantiate_works() {
        let mut app = mock_app_builder_init_funds(&[]);
        let splits_id = app.store_code(contract_splits());

        // make a simple group
        let group_addr = instantiate_group(&mut app, vec![member(owner().to_string(), 0)]);

        // Zero weight fails
        let instantiate_msg = InstantiateMsg {
            group: Group::Cw4Address(group_addr.to_string()),
            admin: None,
        };
        let err = app
            .instantiate_contract(
                splits_id,
                owner(),
                &instantiate_msg,
                &[],
                "greater than zero required total weight",
                None,
            )
            .unwrap_err();
        assert_eq!(
            ContractError::InvalidWeight { weight: 0 },
            err.downcast().unwrap()
        );

        // Single member group with weight is valid
        let group_addr = instantiate_group(&mut app, vec![member(owner().to_string(), 1)]);

        let instantiate_msg = InstantiateMsg {
            group: Group::Cw4Address(group_addr.to_string()),
            admin: None,
        };
        let splits_addr = app
            .instantiate_contract(
                splits_id,
                owner(),
                &instantiate_msg,
                &[],
                "single member group with weight is valid",
                None,
            )
            .unwrap();

        // Verify contract version set properly
        let version = query_contract_info(&app.wrap(), splits_addr.clone()).unwrap();
        assert_eq!(
            ContractVersion {
                contract: "crates.io:sg-splits".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            version,
        );

        // Get members query
        let members: MemberListResponse = app
            .wrap()
            .query_wasm_smart(
                &splits_addr,
                &QueryMsg::ListMembers {
                    start_after: None,
                    limit: None,
                },
            )
            .unwrap();
        assert_eq!(
            members.members,
            vec![Member {
                addr: owner().to_string(),
                weight: 1
            }]
        );
    }

    mod distribute {
        use cosmwasm_std::{coins, Uint128};
        use cw4::Cw4Contract;
        use cw_multi_test::{BankSudo, SudoMsg};

        use super::*;
        use sg_splits::msg::{ExecuteMsg, QueryMsg};

        #[test]
        fn distribute_zero_funds() {
            let mut app = mock_app_builder_init_funds(&[]);

            let (splits_addr, _) = setup_test_case(&mut app, vec![], false);

            let msg = ExecuteMsg::Distribute { denom_list: None };

            let err = app
                .execute_contract(owner(), splits_addr, &msg, &[])
                .unwrap_err();
            assert_eq!(ContractError::NoFunds {}, err.downcast().unwrap());
        }

        #[test]
        fn distribute_non_member() {
            const DENOM: &str = "ugaze";
            let init_funds = coins(100, DENOM);
            let mut app = mock_app_builder_init_funds(&init_funds);

            let (splits_addr, _) = setup_test_case(&mut app, init_funds, false);

            let msg = ExecuteMsg::Distribute { denom_list: None };

            app.execute_contract(
                Addr::unchecked("non_member".to_string()),
                splits_addr,
                &msg,
                &[],
            )
            .unwrap_err();
        }

        #[test]
        fn distribute() {
            const DENOM: &str = "ugaze";
            let init_funds = coins(100, DENOM);
            let mut app = mock_app_builder_init_funds(&init_funds);

            let (splits_addr, _) = setup_test_case_with_internal_group(&mut app, init_funds);

            let msg = ExecuteMsg::Distribute { denom_list: None };

            app.execute_contract(owner(), splits_addr.clone(), &msg, &[])
                .unwrap();

            // make sure the contract doesn't have a balance
            let bal = app.wrap().query_all_balances(splits_addr.clone()).unwrap();
            assert_eq!(bal, &[]);

            // verify amounts for each member - compare by address rather than relying on order
            let msg = QueryMsg::ListMembers {
                start_after: None,
                limit: None,
            };
            let list: MemberListResponse = app.wrap().query_wasm_smart(splits_addr, &msg).unwrap();
            // Map expected balances by address based on weight
            let expected_by_addr = vec![
                (owner(), Uint128::new(50)),
                (member1(), Uint128::new(25)),
                (member2(), Uint128::new(20)),
                (member3(), Uint128::new(5)),
            ];
            for (addr, expected_bal) in expected_by_addr.iter() {
                let bal = app.wrap().query_balance(addr.to_string(), DENOM).unwrap();
                assert_eq!(bal.amount, *expected_bal, "Balance mismatch for {}", addr);
            }
            // Also verify the total number of members
            assert_eq!(list.members.len(), 4);
        }

        #[test]
        fn distribute_under_funded() {
            const DENOM: &str = "ugaze";
            let init_funds = coins(79, DENOM);
            let mut app = mock_app_builder_init_funds(&init_funds);

            let (splits_addr, group_addr) =
                setup_test_case_with_internal_group(&mut app, init_funds);
            let total_weight = Cw4Contract(group_addr).total_weight(&app.wrap()).unwrap();

            let msg = ExecuteMsg::Distribute { denom_list: None };

            let err = app
                .execute_contract(owner(), splits_addr, &msg, &[])
                .unwrap_err();

            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::NotEnoughFunds { min: total_weight }.to_string()
            );
        }

        #[test]
        fn distribute_amount_with_remaining_balance() {
            const DENOM: &str = "ugaze";
            let init_funds = coins(479, DENOM);
            let mut app = mock_app_builder_init_funds(&init_funds);

            let (splits_addr, group_addr) =
                setup_test_case_with_internal_group(&mut app, init_funds.clone());
            let total_weight = Cw4Contract(group_addr).total_weight(&app.wrap()).unwrap();
            let multiplier = init_funds[0].amount / Uint128::from(total_weight);
            let contract_balance = init_funds[0].amount - multiplier * Uint128::from(total_weight);

            let msg = ExecuteMsg::Distribute { denom_list: None };

            let _ = app
                .execute_contract(owner(), splits_addr.clone(), &msg, &[])
                .unwrap();

            // contract has a balance
            let bal = app.wrap().query_all_balances(splits_addr.clone()).unwrap();
            assert_eq!(bal, coins(contract_balance.u128(), DENOM));

            // verify amounts for each member
            let msg = QueryMsg::ListMembers {
                start_after: None,
                limit: None,
            };

            let list: MemberListResponse = app.wrap().query_wasm_smart(splits_addr, &msg).unwrap();
            for member in list.members.iter() {
                let bal = app
                    .wrap()
                    .query_balance(member.addr.to_string(), DENOM)
                    .unwrap();
                assert_eq!(bal.amount, Uint128::from(member.weight) * multiplier)
            }
        }

        #[test]
        fn distribute_with_too_many_members() {
            const DENOM: &str = "ugaze";
            let init_funds = coins(255, DENOM);
            let mut app = mock_app_builder_init_funds(&init_funds);

            let (splits_addr, _) = setup_test_case_with_overflow_group(&mut app, init_funds);

            let msg = ExecuteMsg::Distribute { denom_list: None };
            let err = app
                .execute_contract(owner(), splits_addr, &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::InvalidMemberCount {
                    count: (MAX_GROUP_SIZE + 1) as usize
                }
                .to_string()
            );
        }

        #[test]
        fn distribute_with_zero_weight_members() {
            const DENOM: &str = "ugaze";
            let init_funds = coins(255, DENOM);
            let mut app = mock_app_builder_init_funds(&init_funds);

            let (splits_addr, group_addr) =
                setup_test_case_with_internal_group(&mut app, init_funds);

            let member100 = "member0100".into_addr();
            let member101 = "member0101".into_addr();
            let msg = Cw4GroupExecuteMsg::UpdateMembers {
                remove: vec![],
                add: vec![
                    member(member100.to_string(), 0),
                    member(member101.to_string(), 0),
                ],
            };
            let _ = app
                .execute_contract(owner(), group_addr, &msg, &[])
                .unwrap();

            let msg = ExecuteMsg::Distribute { denom_list: None };
            let _ = app
                .execute_contract(owner(), splits_addr, &msg, &[])
                .unwrap();

            // confirm zero weight members have no balance
            let bal = app.wrap().query_balance(member100.clone(), DENOM).unwrap();
            assert_eq!(bal.amount, Uint128::zero());
            let bal = app.wrap().query_balance(member101.clone(), DENOM).unwrap();
            assert_eq!(bal.amount, Uint128::zero());
        }

        #[test]
        fn distribute_with_group_changes() {
            const DENOM: &str = "ugaze";
            let init_funds = coins(199, DENOM);
            let mut app = mock_app_builder_init_funds(&init_funds);

            let (splits_addr, group_addr) = setup_test_case(&mut app, init_funds.clone(), false);
            let total_weight = Cw4Contract(group_addr.clone())
                .total_weight(&app.wrap())
                .unwrap();
            let multiplier = init_funds[0].amount / Uint128::from(total_weight);
            let contract_balance = init_funds[0].amount - multiplier * Uint128::from(total_weight);
            let mut payouts: HashMap<String, Uint128> = HashMap::new();

            let msg = ExecuteMsg::Distribute { denom_list: None };

            let _ = app
                .execute_contract(owner(), splits_addr.clone(), &msg, &[])
                .unwrap();

            // contract has a balance
            let bal = app.wrap().query_all_balances(splits_addr.clone()).unwrap();
            assert_eq!(bal, coins(contract_balance.u128(), DENOM));

            // verify amounts for each member
            let msg = QueryMsg::ListMembers {
                start_after: None,
                limit: None,
            };

            let list: MemberListResponse = app
                .wrap()
                .query_wasm_smart(splits_addr.clone(), &msg)
                .unwrap();
            for member in list.members.iter() {
                let bal = app
                    .wrap()
                    .query_balance(member.addr.to_string(), DENOM)
                    .unwrap();
                payouts.insert(member.addr.clone(), bal.amount);
                assert_eq!(bal.amount, Uint128::from(member.weight) * multiplier)
            }

            // add members to group
            let member100 = "member0100".into_addr();
            let member101 = "member0101".into_addr();
            let msg = Cw4GroupExecuteMsg::UpdateMembers {
                remove: vec![],
                add: vec![
                    member(member100.to_string(), 2),
                    member(member101.to_string(), 23),
                ],
            };
            let _ = app
                .execute_contract(owner(), group_addr.clone(), &msg, &[])
                .unwrap();
            payouts.insert(member100.to_string(), Uint128::zero());
            payouts.insert(member101.to_string(), Uint128::zero());

            // confirm members were added
            let mem = Cw4Contract(group_addr.clone())
                .is_member(&app.wrap(), &member100, None)
                .unwrap();
            assert!(mem.is_some());
            let mem = Cw4Contract(group_addr.clone())
                .is_member(&app.wrap(), &member101, None)
                .unwrap();
            assert!(mem.is_some());

            // add more funds from bank module to contract
            let more_funds = coins(12345u128, DENOM);
            app.sudo(SudoMsg::Bank({
                BankSudo::Mint {
                    to_address: splits_addr.to_string(),
                    amount: more_funds.clone(),
                }
            }))
            .map_err(|err| println!("{err:?}"))
            .ok();

            // confirm new balance matches
            let bal = app.wrap().query_all_balances(splits_addr.clone()).unwrap();
            assert_eq!(
                bal,
                coins(contract_balance.u128() + more_funds[0].amount.u128(), DENOM)
            );

            // distribute again and check accounting
            let msg = ExecuteMsg::Distribute { denom_list: None };
            let _ = app
                .execute_contract(owner(), splits_addr.clone(), &msg, &[])
                .unwrap();

            // contract has a balance
            let new_total_weight = Cw4Contract(group_addr).total_weight(&app.wrap()).unwrap();
            let new_multiplier =
                (contract_balance + more_funds[0].amount) / Uint128::from(new_total_weight);
            let new_contract_balance = (contract_balance + more_funds[0].amount)
                - new_multiplier * Uint128::from(new_total_weight);
            let bal = app.wrap().query_all_balances(splits_addr.clone()).unwrap();
            assert_eq!(bal, coins(new_contract_balance.u128(), DENOM));

            // confirm member balances
            let msg = QueryMsg::ListMembers {
                start_after: None,
                limit: None,
            };
            let list: MemberListResponse = app.wrap().query_wasm_smart(splits_addr, &msg).unwrap();
            for member in list.members.iter() {
                let bal = app
                    .wrap()
                    .query_balance(member.addr.to_string(), DENOM)
                    .unwrap();
                let prev_payout = payouts
                    .get(&member.addr)
                    .copied()
                    .unwrap_or(Uint128::zero());
                assert_eq!(
                    bal.amount,
                    prev_payout + Uint128::from(member.weight) * new_multiplier,
                    "Balance mismatch for member {}",
                    member.addr
                )
            }
        }
    }
}
