
use std::panic::PanicInfo;

use cw_multi_test::{Contract, ContractWrapper, Executor};
use cosmwasm_std::{Addr, Querier, Binary, StdResult};
use sg_multi_test::StargazeApp;
// use contract::
use sg_std::StargazeMsgWrapper;

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

use crate::error::ContractError;

const OWNER: &str = "admin0001";

fn custom_mock_app() -> StargazeApp {
    StargazeApp::default()
}

pub fn contract() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

fn get_instantiate_contract <'a> () -> StargazeApp {
    let mut app =  custom_mock_app();
    let sg_eth_id = app.store_code(contract());
    assert_eq!(sg_eth_id, 1);
    let msg: InstantiateMsg = InstantiateMsg {
        config: Some("test".to_string()),
    };
    app
        .instantiate_contract(
            sg_eth_id,
            Addr::unchecked(OWNER),
            &msg,
            &[],
            "sg-eg-airdrop",
            Some(Addr::unchecked(OWNER).to_string()),
        )
        .unwrap();
    app
}
    

#[test]
fn test_instantiate() {
    get_instantiate_contract();
}

#[test]
fn test_not_authorized_add_eth() {
    let mut app = get_instantiate_contract();
    let sg_eth_addr = Addr::unchecked("contract0".to_string());

    let fake_admin = Addr::unchecked("fake");
    let eth_address = Addr::unchecked("testing_addr");
    let execute_msg = ExecuteMsg::AddEligibleEth {
        eth_address: eth_address.to_string(),
    };
    let res = app.execute_contract(fake_admin, sg_eth_addr, &execute_msg, &[]);
    let error = res.unwrap_err();
    let expected_err_msg = "Unauthorized admin, sender is fake"; 
    assert_eq!(error.root_cause().to_string(), expected_err_msg)
}



#[test]
fn test_authorized_add_eth() {
    let mut app = get_instantiate_contract();
    let sg_eth_addr = Addr::unchecked("contract0".to_string());

    let true_admin = Addr::unchecked(OWNER);
    let eth_address = Addr::unchecked("testing_addr");
    let execute_msg = ExecuteMsg::AddEligibleEth {
        eth_address: eth_address.to_string(),
    };
    let res = app.execute_contract(
        true_admin, sg_eth_addr, &execute_msg, &[]);
    match res {
        Ok(_) => (),
        Err(_) => panic!("Could not add eth address")
    }

}

#[test]
fn test_add_eth_and_verify() {
    let mut app = get_instantiate_contract();
    let sg_eth_addr = Addr::unchecked("contract0".to_string());

    let true_admin = Addr::unchecked(OWNER);
    let eth_address = Addr::unchecked("testing_addr");
    let execute_msg = ExecuteMsg::AddEligibleEth {
        eth_address: eth_address.to_string(),
    };
    let res = app.execute_contract(
        true_admin, sg_eth_addr.clone(), &execute_msg, &[]);
    
    let query_msg = QueryMsg::AirdropEligible { eth_address: eth_address};
    let result: StdResult<Binary> = app.wrap().query_wasm_smart(sg_eth_addr, &query_msg);
    println!("query response {:?}", result.unwrap_err());
    assert!(false)

}
