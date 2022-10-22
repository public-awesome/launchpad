use cosmwasm_std::Addr;
use cw_multi_test::{Contract, ContractWrapper, Executor};
use sg_multi_test::StargazeApp;
// use contract::
use sg_std::StargazeMsgWrapper;

use crate::msg::InstantiateMsg;

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

#[test]
fn test_instantiate() {
    let mut app = custom_mock_app();
    let sg_eth_id = app.store_code(contract());
    assert_eq!(sg_eth_id, 1);

    let msg: InstantiateMsg = InstantiateMsg {
        config: Some("test".to_string()),
    };

    app.instantiate_contract(
        sg_eth_id,
        Addr::unchecked(OWNER),
        &msg,
        &[],
        "sg-eg-airdrop",
        Some(Addr::unchecked(OWNER).to_string()),
    )
    .unwrap();
}
