use crate::common_setup::setup_accounts_and_block::setup_accounts;
use crate::common_setup::setup_collection_whitelist::setup_whitelist_contract;

use crate::common_setup::contract_boxes::{contract_vending_factory, App};
use crate::common_setup::setup_minter::vending_minter::mock_params::{
    mock_create_minter, mock_params,
};
use crate::sg_eth_airdrop::constants::collection_constants::CREATION_FEE;
use crate::sg_eth_airdrop::setup::mock_minter_contract::mock_minter;
use cosmwasm_std::{coins, Addr, Timestamp};
use cw_multi_test::Executor;
use sg2::msg::Sg2ExecuteMsg;
use sg2::tests::mock_collection_params_1;

use sg_utils::{GENESIS_MINT_START_TIME, NATIVE_DENOM};

use crate::sg_eth_airdrop::setup::mock_whitelist_contract::mock_whitelist;

fn configure_mock_minter(app: &mut App, creator: Addr) -> Addr {
    let minter_code_id = app.store_code(mock_minter());

    println!("minter_code_id: {minter_code_id}");
    let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);

    let factory_code_id = app.store_code(contract_vending_factory());
    println!("factory_code_id: {factory_code_id}");

    let mut params = mock_params(None);
    params.code_id = minter_code_id;

    let factory_addr = app
        .instantiate_contract(
            factory_code_id,
            creator.clone(),
            &vending_factory::msg::InstantiateMsg { params },
            &[],
            "factory",
            None,
        )
        .unwrap();
    let start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    let collection_params = mock_collection_params_1(Some(start_time));
    let msg = mock_create_minter(None, collection_params, None);
    let msg = Sg2ExecuteMsg::CreateMinter(msg);
    let res = app
        .execute_contract(creator, factory_addr, &msg, &creation_fee)
        .unwrap();
    // Extract minter address from response events
    let minter_addr = res
        .events
        .iter()
        .find(|e| e.ty == "instantiate")
        .and_then(|e| e.attributes.iter().find(|a| a.key == "_contract_address"))
        .map(|a| Addr::unchecked(&a.value))
        .expect("minter address not found");
    minter_addr
}
pub fn configure_mock_minter_with_mock_whitelist(app: &mut App) -> Addr {
    let (creator, _) = setup_accounts(app);
    let minter_addr = configure_mock_minter(app, creator.clone());
    let whitelist_code_id = app.store_code(mock_whitelist());
    let whitelist_addr = setup_whitelist_contract(app, &creator, Some(whitelist_code_id), None);
    // Set whitelist on minter
    let set_whitelist_msg = vending_minter::msg::ExecuteMsg::SetWhitelist {
        whitelist: whitelist_addr.to_string(),
    };
    app.execute_contract(creator, minter_addr.clone(), &set_whitelist_msg, &[])
        .unwrap();
    minter_addr
}
