#[cfg(test)]
mod tests {
    use crate::{
        msg::{InstantiateMsg, QueryMsg},
        ContractError,
    };
    use cosmwasm_std::{Addr, Coin};
    use cw2::{query_contract_info, ContractVersion};
    use cw4::{Member, MemberListResponse};
    use cw_multi_test::{next_block, Contract, ContractWrapper, Executor};
    use sg_multi_test::StargazeApp;
    use sg_std::StargazeMsgWrapper;

    const OWNER: &str = "admin0001";
    const MEMBER1: &str = "member0001";
    const MEMBER2: &str = "member0002";
    const MEMBER3: &str = "member0003";

    fn member<T: Into<String>>(addr: T, weight: u64) -> Member {
        Member {
            addr: addr.into(),
            weight,
        }
    }

    fn members() -> Vec<Member> {
        vec![
            member(OWNER, 50),
            member(MEMBER1, 25),
            member(MEMBER2, 20),
            member(MEMBER3, 5),
        ]
    }

    pub fn contract_splits() -> Box<dyn Contract<StargazeMsgWrapper>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        )
        .with_reply(crate::contract::reply);

        Box::new(contract)
    }

    pub fn contract_group() -> Box<dyn Contract<StargazeMsgWrapper>> {
        let contract = ContractWrapper::new_with_empty(
            cw4_group::contract::execute,
            cw4_group::contract::instantiate,
            cw4_group::contract::query,
        );
        Box::new(contract)
    }

    #[track_caller]
    fn instantiate_splits(app: &mut StargazeApp, members: Vec<Member>) -> Addr {
        let splits_id = app.store_code(contract_splits());
        let group_id = app.store_code(contract_group());

        let msg = crate::msg::InstantiateMsg {
            group_code_id: group_id,
            members,
            group_admin: None,
        };
        app.instantiate_contract(splits_id, Addr::unchecked(OWNER), &msg, &[], "splits", None)
            .unwrap()
    }

    #[track_caller]
    fn setup_test_case(
        app: &mut StargazeApp,
        init_funds: Vec<Coin>,
        _multisig_as_group_admin: bool,
    ) -> Addr {
        // Instantiate group contract with members (and OWNER as admin) + Set up Splits backed by this group
        let splits_addr = instantiate_splits(app, members());
        app.update_block(next_block);

        // Bonus: set some funds on the splits contract for future proposals
        if !init_funds.is_empty() {
            app.send_tokens(Addr::unchecked(OWNER), splits_addr.clone(), &init_funds)
                .unwrap();
        }
        splits_addr
    }

    #[test]
    fn test_instantiate_works() {
        let mut app = StargazeApp::mock_app(&[], Addr::unchecked("unused"));
        let splits_id = app.store_code(contract_splits());
        let group_id = app.store_code(contract_group());

        // Zero weight fails
        let instantiate_msg = InstantiateMsg {
            group_code_id: group_id,
            members: vec![member(OWNER, 0)],
            group_admin: None,
        };
        let err = app
            .instantiate_contract(
                splits_id,
                Addr::unchecked(OWNER),
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
        let instantiate_msg = InstantiateMsg {
            group_code_id: group_id,
            members: vec![member(OWNER, 1)],
            group_admin: None,
        };
        let splits_addr = app
            .instantiate_contract(
                splits_id,
                Addr::unchecked(OWNER),
                &instantiate_msg,
                &[],
                "single member group with weight is valid",
                None,
            )
            .unwrap();

        // Verify contract version set properly
        let version = query_contract_info(&app, splits_addr.clone()).unwrap();
        assert_eq!(
            ContractVersion {
                contract: "crates.io:sg-splits".to_string(),
                version: "0.1.0".to_string(),
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
                addr: OWNER.into(),
                weight: 1
            }]
        );
    }

    mod distribute {
        use cosmwasm_std::{coins, Uint128};

        use super::*;
        use crate::msg::{ExecuteMsg, QueryMsg};

        #[test]
        fn distribute_zero_funds() {
            let mut app = StargazeApp::mock_app(&[], Addr::unchecked("unused"));

            let splits_addr = setup_test_case(&mut app, vec![], false);

            let msg = ExecuteMsg::Distribute {};

            let err = app
                .execute_contract(Addr::unchecked(OWNER), splits_addr, &msg, &[])
                .unwrap_err();
            assert_eq!(ContractError::NoFunds {}, err.downcast().unwrap());
        }

        #[test]
        fn distribute_non_member() {
            const DENOM: &str = "ustars";
            let init_funds = coins(100, DENOM);
            let mut app = StargazeApp::mock_app(&init_funds, Addr::unchecked(OWNER));

            let splits_addr = setup_test_case(&mut app, init_funds, false);

            let msg = ExecuteMsg::Distribute {};

            app.execute_contract(
                Addr::unchecked("non_memeber".to_string()),
                splits_addr,
                &msg,
                &[],
            )
            .unwrap_err();
        }

        #[test]
        fn distribute() {
            const DENOM: &str = "ustars";
            let init_funds = coins(100, DENOM);
            let mut app = StargazeApp::mock_app(&init_funds, Addr::unchecked(OWNER));

            let splits_addr = setup_test_case(&mut app, init_funds, false);

            let msg = ExecuteMsg::Distribute {};

            app.execute_contract(Addr::unchecked(OWNER), splits_addr.clone(), &msg, &[])
                .unwrap();

            // make sure the contract doesn't have a balance
            let bal = app.wrap().query_all_balances(splits_addr.clone()).unwrap();
            assert_eq!(bal, &[]);

            // verify amounts for each member
            let msg = QueryMsg::ListMembers {
                start_after: None,
                limit: None,
            };
            let list: MemberListResponse = app.wrap().query_wasm_smart(splits_addr, &msg).unwrap();
            let mut expected_balances = vec![
                Uint128::new(5),
                Uint128::new(20),
                Uint128::new(25),
                Uint128::new(50),
            ];
            for member in list.members.iter() {
                let bal = app
                    .wrap()
                    .query_balance(member.addr.to_string(), DENOM)
                    .unwrap();
                assert_eq!(bal.amount, expected_balances.pop().unwrap())
            }
        }
    }
}
