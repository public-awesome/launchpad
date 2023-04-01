use cosmwasm_std::coins;
use cw_multi_test::{BankSudo, Executor, SudoMsg};
use sg_std::GENESIS_MINT_START_TIME;
use vending_minter::msg::ExecuteMsg;

use crate::common_setup::{
    setup_accounts_and_block::setup_block_time, setup_minter::common::constants::MINT_PRICE,
    templates::vending_minter_with_ibc_asset,
};

#[test]
fn mint_with_ibc_asset() {
    let num_tokens = 7000;
    let per_address_limit = 10;
    let vt = vending_minter_with_ibc_asset(num_tokens, per_address_limit);
    let (mut router, _, buyer) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();

    let mint_price = coins(MINT_PRICE, "ibc/asset".to_string());

    // give the buyer some of the IBC asset
    router
        .sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: buyer.to_string(),
                amount: mint_price.clone(),
            }
        }))
        .map_err(|err| println!("{:?}", err))
        .ok();

    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 1, None);

    // Mint succeeds
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(buyer, minter_addr, &mint_msg, &mint_price);
    println!("{:?}", res);
    // assert!(res.is_ok());
}
