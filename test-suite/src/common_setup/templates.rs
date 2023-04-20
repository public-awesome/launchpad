use crate::common_setup::contract_boxes::{contract_sg721_base, custom_mock_app};
use crate::common_setup::msg::MinterTemplateResponse;
use crate::common_setup::{
    msg::MinterCollectionResponse,
    setup_accounts_and_block::setup_accounts,
    setup_minter::common::minter_params::minter_params_token,
    setup_minter::vending_minter::setup::{configure_minter, vending_minter_code_ids},
};

use crate::common_setup::setup_minter::base_minter::setup::base_minter_sg721_nt_code_ids;
use crate::common_setup::setup_minter::base_minter::setup::configure_base_minter;
use cosmwasm_std::{coin, Coin, coins, Timestamp, Uint128};
use cw_multi_test::{AppResponse, Contract, Executor};
use sg2::tests::mock_collection_params_1;
use sg_multi_test::StargazeApp;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM, StargazeMsgWrapper};
use open_edition_factory::types::{NftData, NftMetadataType};
use sg2::msg::Sg2ExecuteMsg;
use crate::common_setup::setup_minter::common::constants::{CREATION_FEE, MIN_MINT_PRICE_OPEN_EDITION};
use crate::common_setup::setup_minter::common::parse_response::build_collection_response;
use crate::common_setup::setup_minter::open_edition_minter::mock_params::{mock_create_minter, mock_params_proper};
use crate::common_setup::setup_minter::open_edition_minter::setup::open_edition_minter_code_ids;

use super::msg::Accounts;
use super::setup_minter::base_minter::setup::base_minter_sg721_collection_code_ids;
use super::setup_minter::common::constants::MIN_MINT_PRICE;
use super::setup_minter::common::minter_params::minter_params_all;

pub fn vending_minter_template(num_tokens: u32) -> MinterTemplateResponse<Accounts> {
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
    MinterTemplateResponse {
        router: app,
        collection_response_vec: minter_collection_response,
        accts: Accounts { creator, buyer },
    }
}

pub fn vending_minter_per_address_limit(
    num_tokens: u32,
    per_address_limit: u32,
) -> MinterTemplateResponse<Accounts> {
    let mut app = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut app);
    let start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    let collection_params = mock_collection_params_1(Some(start_time));
    let init_msg = vending_factory::msg::VendingMinterInitMsgExtension {
        base_token_uri: "ipfs://aldkfjads".to_string(),
        payment_address: None,
        start_time: Timestamp::from_nanos(GENESIS_MINT_START_TIME),
        num_tokens,
        mint_price: coin(MIN_MINT_PRICE, NATIVE_DENOM),
        per_address_limit,
        whitelist: Some("invalid address".to_string()),
    };

    let minter_params = minter_params_all(num_tokens, None, None, Some(init_msg));
    let code_ids = vending_minter_code_ids(&mut app);
    let minter_collection_response: Vec<MinterCollectionResponse> = configure_minter(
        &mut app,
        creator.clone(),
        vec![collection_params],
        vec![minter_params],
        code_ids,
    );
    MinterTemplateResponse {
        router: app,
        collection_response_vec: minter_collection_response,
        accts: Accounts { creator, buyer },
    }
}

pub fn vending_minter_with_start_time(
    num_tokens: u32,
    start_time: Timestamp,
) -> MinterTemplateResponse<Accounts> {
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
    MinterTemplateResponse {
        router: app,
        collection_response_vec: minter_collection_response,
        accts: Accounts { creator, buyer },
    }
}

pub fn vending_minter_with_app(
    num_tokens: u32,
    mut app: StargazeApp,
) -> MinterTemplateResponse<Accounts> {
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
    MinterTemplateResponse {
        router: app,
        collection_response_vec: minter_collection_response,
        accts: Accounts { creator, buyer },
    }
}

pub fn vending_minter_with_specified_sg721(
    num_tokens: u32,
    sg721_code_id: u64,
) -> MinterTemplateResponse<Accounts> {
    let mut app = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut app);
    let start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    let collection_params = mock_collection_params_1(Some(start_time));
    let minter_params = minter_params_token(num_tokens);
    let mut code_ids = vending_minter_code_ids(&mut app);
    code_ids.sg721_code_id = sg721_code_id;
    let minter_collection_response: Vec<MinterCollectionResponse> = configure_minter(
        &mut app,
        creator.clone(),
        vec![collection_params],
        vec![minter_params],
        code_ids,
    );
    MinterTemplateResponse {
        router: app,
        collection_response_vec: minter_collection_response,
        accts: Accounts { creator, buyer },
    }
}

pub fn base_minter_with_sg721nt(num_tokens: u32) -> MinterTemplateResponse<Accounts> {
    let mut router = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut router);
    let start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    let collection_params = mock_collection_params_1(Some(start_time));
    let minter_params = minter_params_token(num_tokens);
    let code_ids = base_minter_sg721_nt_code_ids(&mut router);
    let minter_collection_response = configure_base_minter(
        &mut router,
        creator.clone(),
        vec![collection_params],
        vec![minter_params],
        code_ids,
    );
    MinterTemplateResponse {
        router,
        collection_response_vec: minter_collection_response,
        accts: Accounts { creator, buyer },
    }
}

pub fn base_minter_with_sg721(num_tokens: u32) -> MinterTemplateResponse<Accounts> {
    let mut router = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut router);
    let start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    let collection_params = mock_collection_params_1(Some(start_time));
    let minter_params = minter_params_token(num_tokens);
    let code_ids = base_minter_sg721_collection_code_ids(&mut router);
    let minter_collection_response = configure_base_minter(
        &mut router,
        creator.clone(),
        vec![collection_params],
        vec![minter_params],
        code_ids,
    );
    MinterTemplateResponse {
        router,
        collection_response_vec: minter_collection_response,
        accts: Accounts { creator, buyer },
    }
}

pub fn base_minter_with_specified_sg721(
    num_tokens: u32,
    sg721_code_id: u64,
) -> MinterTemplateResponse<Accounts> {
    let mut router = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut router);
    let start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    let collection_params = mock_collection_params_1(Some(start_time));
    let minter_params = minter_params_token(num_tokens);
    let mut code_ids = base_minter_sg721_collection_code_ids(&mut router);
    code_ids.sg721_code_id = sg721_code_id;
    let minter_collection_response = configure_base_minter(
        &mut router,
        creator.clone(),
        vec![collection_params],
        vec![minter_params],
        code_ids,
    );
    MinterTemplateResponse {
        router,
        collection_response_vec: minter_collection_response,
        accts: Accounts { creator, buyer },
    }
}

// Custom params set to a high level function to ease the tests
#[allow(clippy::too_many_arguments)]
pub fn open_edition_minter_custom_template(
    start_minter_time: Option<Timestamp>,
    end_minter_time: Option<Timestamp>,
    nft_data: Option<NftData>,
    abs_max_mint_per_address: Option<u32>,
    per_address_limit_minter: Option<u32>,
    mint_price_minter: Option<Coin>,
    sg721_code: Option<Box<dyn Contract<StargazeMsgWrapper>>>,
    sg721_codeid: Option<u64>
) -> Result<MinterTemplateResponse<Accounts>, anyhow::Result<AppResponse>>  {
    let mut app = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut app);
    let code_ids = open_edition_minter_code_ids(&mut app, sg721_code.unwrap_or(contract_sg721_base()));

    // Factory params
    let mut factory_params = mock_params_proper();
    factory_params.extension.abs_max_mint_per_address = abs_max_mint_per_address.unwrap_or(factory_params.extension.abs_max_mint_per_address);

    let factory_addr = app
        .instantiate_contract(
            code_ids.factory_code_id,
            creator.clone(),
            &open_edition_factory::msg::InstantiateMsg { params: factory_params },
            &[],
            "factory",
            None,
        );

    let factory_addr = factory_addr.unwrap();

    // Minter -> Default params
    let start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 100);
    let end_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10_000);
    let per_address_limit_minter = per_address_limit_minter.or(Some(1));
    let mint_price = mint_price_minter.or(Some(Coin {
        denom: NATIVE_DENOM.to_string(),
        amount: Uint128::new(MIN_MINT_PRICE_OPEN_EDITION)
    }));
    let collection_params = mock_collection_params_1(Some(start_time));
    let default_nft_data = nft_data.unwrap_or(NftData {
        nft_data_type: NftMetadataType::OffChainMetadata,
        token_id_prefix: "Token ID #".to_string(),
        extension: None,
        token_uri: Some("ipfs://bafybeiavall5udkxkdtdm4djezoxrmfc6o5fn2ug3ymrlvibvwmwydgrkm/1.jpg".to_string()),
    });
    let mut msg = mock_create_minter(
        start_minter_time.or(Some(start_time)),
        end_minter_time.or(Some(end_time)),
        mint_price,
        per_address_limit_minter,
        default_nft_data,
        collection_params,
        None
    );
    msg.collection_params.code_id = sg721_codeid.unwrap_or(3);
    msg.collection_params.info.creator = creator.to_string();
    let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);
    let msg = Sg2ExecuteMsg::CreateMinter(msg);

    let res = app.execute_contract(
        creator.clone(),
        factory_addr.clone(),
        &msg,
        &creation_fee
    );
    if res.is_err() {
        return Err(res)
    }

    let minter_collection_res = build_collection_response(res, factory_addr);

    Ok(MinterTemplateResponse {
        router: app,
        collection_response_vec: vec![minter_collection_res],
        accts: Accounts { creator, buyer },
    })
}
