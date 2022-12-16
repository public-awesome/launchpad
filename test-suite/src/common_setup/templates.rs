use crate::common_setup::contract_boxes::custom_mock_app;
use crate::common_setup::msg::VendingTemplateResponse;
use crate::common_setup::{
    msg::MinterCollectionResponse,
    setup_accounts_and_block::setup_accounts,
    setup_minter::{configure_minter, minter_params_token, vending_minter_code_ids},
};
use cosmwasm_std::Timestamp;
use sg2::tests::mock_collection_params_1;
use sg_multi_test::StargazeApp;
use sg_std::GENESIS_MINT_START_TIME;

pub fn vending_minter_template(num_tokens: u32) -> VendingTemplateResponse {
    let mut app = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut app);
    let start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    let collection_params = mock_collection_params_1(Some(start_time));
    let minter_params = minter_params_token(num_tokens);
    let code_ids = vending_minter_code_ids(&mut app);
    let minter_collection_response: Vec<MinterCollectionResponse> = configure_minter(
        &mut app,
        creator.clone(),
        vec![collection_params],
        vec![minter_params],
        code_ids,
    );
    VendingTemplateResponse {
        router: app,
        collection_response_vec: minter_collection_response,
        creator,
        buyer,
    }
}

pub fn vending_minter_with_start_time(
    num_tokens: u32,
    start_time: Timestamp,
) -> VendingTemplateResponse {
    let mut app = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut app);
    let collection_params = mock_collection_params_1(Some(start_time));
    let minter_params = minter_params_token(num_tokens);
    let code_ids = vending_minter_code_ids(&mut app);
    let minter_collection_response: Vec<MinterCollectionResponse> = configure_minter(
        &mut app,
        creator.clone(),
        vec![collection_params],
        vec![minter_params],
        code_ids,
    );
    VendingTemplateResponse {
        router: app,
        collection_response_vec: minter_collection_response,
        creator,
        buyer,
    }
}

pub fn vending_minter_with_app(num_tokens: u32, mut app: StargazeApp) -> VendingTemplateResponse {
    let start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    let (creator, buyer) = setup_accounts(&mut app);
    let collection_params = mock_collection_params_1(Some(start_time));
    let minter_params = minter_params_token(num_tokens);
    let code_ids = vending_minter_code_ids(&mut app);
    let minter_collection_response: Vec<MinterCollectionResponse> = configure_minter(
        &mut app,
        creator.clone(),
        vec![collection_params],
        vec![minter_params],
        code_ids,
    );
    VendingTemplateResponse {
        router: app,
        collection_response_vec: minter_collection_response,
        creator,
        buyer,
    }
}
