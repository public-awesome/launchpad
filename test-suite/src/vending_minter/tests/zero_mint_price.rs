use crate::common_setup::contract_boxes::custom_mock_app;
use crate::common_setup::msg::{MinterCollectionResponse, MinterSetupParams};
use crate::common_setup::setup_accounts_and_block::setup_accounts;
use crate::common_setup::setup_minter::common::minter_params::minter_params_all;
use crate::common_setup::setup_minter::vending_minter::setup::{
    configure_minter_zero_mint_price, vending_minter_code_ids,
};
use crate::common_setup::templates::vending_minter_zero_mint_price;
use crate::common_setup::{
    setup_accounts_and_block::coins_for_msg, setup_accounts_and_block::setup_block_time,
};
use cosmwasm_std::{coin, coins, Coin, Timestamp, Uint128};
use cw721::{Cw721QueryMsg, OwnerOfResponse};
use cw_multi_test::Executor;
use sg2::tests::mock_collection_params_1;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
use vending_minter::msg::MintCountResponse;
use vending_minter::msg::{ExecuteMsg, QueryMsg, StartTimeResponse};

const INITIAL_BALANCE: u128 = 2_000_000_000;
const MINT_PRICE: u128 = 0;
const MINT_FEE: u128 = 10_000_000;
const ADMIN_MINT_PRICE: u128 = 0;

#[test]
fn zero_mint_price() {
    let num_tokens = 2;
    let mut app = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut app);
    let start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    let collection_params = mock_collection_params_1(Some(start_time));
    let init_msg = vending_factory::msg::VendingMinterInitMsgExtension {
        base_token_uri: "ipfs://aldkfjads".to_string(),
        payment_address: None,
        start_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME),
        num_tokens,
        mint_price: coin(0u128, NATIVE_DENOM),
        per_address_limit: 1,
        whitelist: Some("invalid address".to_string()),
    };

    let minter_params = minter_params_all(num_tokens, None, None, Some(init_msg.clone()));
    let code_ids = vending_minter_code_ids(&mut app);
    let mut minter_collection_info: Vec<MinterCollectionResponse> = vec![];

    let setup_params: MinterSetupParams = MinterSetupParams {
        router: &mut app,
        minter_admin: creator.clone(),
        num_tokens,
        collection_params,
        splits_addr: None,
        minter_code_id: code_ids.minter_code_id,
        factory_code_id: code_ids.factory_code_id,
        sg721_code_id: code_ids.sg721_code_id,
        start_time: Some(init_msg.start_time),
        init_msg: Some(init_msg.clone()),
    };
}
