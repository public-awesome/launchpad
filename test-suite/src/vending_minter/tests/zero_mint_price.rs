use crate::common_setup::contract_boxes::custom_mock_app;
use crate::common_setup::msg::MinterSetupParams;
use crate::common_setup::setup_accounts_and_block::setup_accounts;
use crate::common_setup::setup_accounts_and_block::setup_block_time;
use crate::common_setup::setup_minter::common::constants::CREATION_FEE;
use crate::common_setup::setup_minter::common::minter_params::minter_params_all;
use crate::common_setup::setup_minter::vending_minter::mock_params::{
    mock_create_minter_init_msg, mock_init_extension, mock_params,
};
use crate::common_setup::setup_minter::vending_minter::setup::vending_minter_code_ids;
use cosmwasm_std::{coin, coins, Addr, Timestamp};
use cw721::TokensResponse;
use cw_multi_test::Executor;
use sg2::msg::Sg2ExecuteMsg;
use sg2::tests::{mock_collection_params, mock_collection_params_1};
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
use vending_minter::msg::ExecuteMsg;

const MINT_PRICE: u128 = 0;

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
        mint_price: coin(MINT_PRICE, NATIVE_DENOM),
        per_address_limit: 1,
        whitelist: Some("invalid address".to_string()),
    };

    let minter_params = minter_params_all(num_tokens, None, None, Some(init_msg));
    let code_ids = vending_minter_code_ids(&mut app);

    let setup_params: MinterSetupParams = MinterSetupParams {
        router: &mut app,
        minter_admin: creator,
        num_tokens,
        collection_params,
        splits_addr: minter_params.splits_addr,
        minter_code_id: code_ids.minter_code_id,
        factory_code_id: code_ids.factory_code_id,
        sg721_code_id: code_ids.sg721_code_id,
        start_time: minter_params.start_time,
        init_msg: minter_params.init_msg,
    };

    let minter_code_id = setup_params.minter_code_id;
    let router = setup_params.router;
    let factory_code_id = setup_params.factory_code_id;
    let sg721_code_id = setup_params.sg721_code_id;
    let minter_admin = setup_params.minter_admin;

    let mut params = mock_params();
    params.code_id = minter_code_id;
    params.min_mint_price = coin(MINT_PRICE, NATIVE_DENOM);

    let factory_addr = router
        .instantiate_contract(
            factory_code_id,
            minter_admin.clone(),
            &vending_factory::msg::InstantiateMsg { params },
            &[],
            "factory",
            None,
        )
        .unwrap();

    let mut init_msg = mock_init_extension(None, None);
    init_msg.mint_price = coin(MINT_PRICE, NATIVE_DENOM);
    let mut msg = mock_create_minter_init_msg(mock_collection_params(), init_msg);
    msg.collection_params.code_id = sg721_code_id;
    msg.collection_params.info.creator = minter_admin.to_string();
    let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);
    let msg = Sg2ExecuteMsg::CreateMinter(msg);

    let res = router.execute_contract(minter_admin, factory_addr, &msg, &creation_fee);
    assert!(res.is_ok());

    setup_block_time(router, GENESIS_MINT_START_TIME + 1, None);

    // Mint succeeds
    let minter_addr = Addr::unchecked("contract1");
    let sg721 = Addr::unchecked("contract2");
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(buyer.clone(), minter_addr, &mint_msg, &[]);
    assert!(res.is_ok());

    // Buyer has 1 token
    let res: TokensResponse = router
        .wrap()
        .query_wasm_smart(
            sg721,
            &sg721_base::msg::QueryMsg::Tokens {
                owner: buyer.to_string(),
                start_after: None,
                limit: None,
            },
        )
        .unwrap();
    assert_eq!(res.tokens.len(), 1);
}
