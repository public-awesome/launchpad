use crate::common_setup::contract_boxes::{custom_mock_app, App};
use crate::common_setup::msg::MinterTemplateResponse;
use crate::common_setup::{
    msg::MinterCollectionResponse,
    setup_accounts_and_block::setup_accounts,
    setup_minter::common::minter_params::minter_params_token,
    setup_minter::vending_minter::setup::{configure_minter, vending_minter_code_ids},
};

use super::msg::{Accounts, CodeIds, MinterTemplateResponseCodeIds};
use super::setup_minter::base_minter::setup::base_minter_sg721_collection_code_ids;
use super::setup_minter::common::constants::{MINT_PRICE, MIN_MINT_PRICE};
use super::setup_minter::common::minter_params::minter_params_all;
use super::setup_minter::open_edition_minter::setup::configure_open_edition_minter;
use crate::common_setup::setup_accounts_and_block::CREATION_FEE;
use crate::common_setup::setup_minter::base_minter::setup::base_minter_sg721_nt_code_ids;
use crate::common_setup::setup_minter::base_minter::setup::configure_base_minter;
use crate::common_setup::setup_minter::open_edition_minter::minter_params::minter_params_open_edition;
use crate::common_setup::setup_minter::open_edition_minter::setup::open_edition_minter_code_ids;
use crate::common_setup::setup_minter::vending_minter::setup::vending_minter_updatable_code_ids;
use cosmwasm_std::{coin, Timestamp};
use cw_multi_test::{AppResponse, BankSudo, SudoMsg};
use open_edition_factory::msg::OpenEditionMinterInitMsgExtension;
use open_edition_factory::state::{OpenEditionMinterParams, ParamsExtension};
use open_edition_factory::types::NftData;
use sg2::tests::{mock_collection_params_1, mock_collection_two};

use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};

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

pub fn vending_minter_with_ibc_asset(
    num_tokens: u32,
    per_address_limit: u32,
    denom: &str,
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
        mint_price: coin(MINT_PRICE, denom),
        per_address_limit,
        whitelist: None,
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

pub fn vending_minter_with_app(num_tokens: u32, mut app: App) -> MinterTemplateResponse<Accounts> {
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

pub fn vending_minter_with_sg721_updatable(num_tokens: u32) -> MinterTemplateResponse<Accounts> {
    let mut app = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut app);
    let start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    let collection_params = mock_collection_params_1(Some(start_time));
    let minter_params = minter_params_token(num_tokens);
    let code_ids = vending_minter_updatable_code_ids(&mut app);
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

pub fn vending_minter_updatable_with_app(
    num_tokens: u32,
    mut app: App,
) -> MinterTemplateResponse<Accounts> {
    let start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    let (creator, buyer) = setup_accounts(&mut app);
    let collection_params = mock_collection_params_1(Some(start_time));
    let minter_params = minter_params_token(num_tokens);
    let code_ids = vending_minter_updatable_code_ids(&mut app);
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

pub fn vending_minter_with_updatable_and_start_time(
    num_tokens: u32,
    start_time: Timestamp,
) -> MinterTemplateResponse<Accounts> {
    let mut app = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut app);
    let collection_params = mock_collection_params_1(Some(start_time));
    let minter_params = minter_params_token(num_tokens);
    let code_ids = vending_minter_updatable_code_ids(&mut app);
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

pub fn open_edition_minter_custom_template(
    params_extension: ParamsExtension,
    init_msg: OpenEditionMinterInitMsgExtension,
) -> Result<MinterTemplateResponseCodeIds<Accounts>, anyhow::Result<AppResponse>> {
    let mut app = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut app);
    let code_ids = open_edition_minter_code_ids(&mut app);
    let collection_params = mock_collection_params_1(None);
    let minter_params =
        minter_params_open_edition(params_extension, init_msg, None, None, None, None, None);

    let minter_collection_response = configure_open_edition_minter(
        &mut app,
        creator.clone(),
        vec![collection_params],
        vec![minter_params],
        code_ids.clone(),
    );
    println!(
        "minter collection response is {:?}",
        minter_collection_response[0].error
    );
    Ok(MinterTemplateResponseCodeIds {
        router: app,
        collection_response_vec: minter_collection_response,
        accts: Accounts { creator, buyer },
        code_ids,
    })
}

pub fn open_edition_minter_nft_data(
    params_extension: ParamsExtension,
    init_msg: OpenEditionMinterInitMsgExtension,
    nft_data: NftData,
) -> Result<MinterTemplateResponse<Accounts>, anyhow::Result<AppResponse>> {
    let mut app = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut app);
    let code_ids = open_edition_minter_code_ids(&mut app);
    let collection_params = mock_collection_params_1(None);
    let minter_params = minter_params_open_edition(
        params_extension,
        init_msg,
        None,
        None,
        None,
        Some(nft_data),
        None,
    );

    let minter_collection_response = configure_open_edition_minter(
        &mut app,
        creator.clone(),
        vec![collection_params],
        vec![minter_params],
        code_ids,
    );
    Ok(MinterTemplateResponse {
        router: app,
        collection_response_vec: minter_collection_response,
        accts: Accounts { creator, buyer },
    })
}

pub fn open_edition_minter_start_and_end_time(
    params_extension: ParamsExtension,
    init_msg: OpenEditionMinterInitMsgExtension,
    start_time: Option<Timestamp>,
    end_time: Option<Timestamp>,
) -> Result<MinterTemplateResponse<Accounts>, anyhow::Result<AppResponse>> {
    let mut app = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut app);
    let code_ids = open_edition_minter_code_ids(&mut app);
    let collection_params = mock_collection_params_1(None);
    let minter_params = minter_params_open_edition(
        params_extension,
        init_msg,
        start_time,
        end_time,
        None,
        None,
        None,
    );

    let minter_collection_response = configure_open_edition_minter(
        &mut app,
        creator.clone(),
        vec![collection_params],
        vec![minter_params],
        code_ids,
    );
    Ok(MinterTemplateResponse {
        router: app,
        collection_response_vec: minter_collection_response,
        accts: Accounts { creator, buyer },
    })
}

pub fn open_edition_minter_custom_code_ids(
    app: App,
    params_extension: ParamsExtension,
    init_msg: OpenEditionMinterInitMsgExtension,
    code_ids: CodeIds,
) -> Result<MinterTemplateResponse<Accounts>, anyhow::Result<AppResponse>> {
    let mut app = app;
    let (creator, buyer) = setup_accounts(&mut app);
    let collection_params = mock_collection_params_1(None);
    let minter_params =
        minter_params_open_edition(params_extension, init_msg, None, None, None, None, None);

    let minter_collection_response = configure_open_edition_minter(
        &mut app,
        creator.clone(),
        vec![collection_params],
        vec![minter_params],
        code_ids,
    );
    Ok(MinterTemplateResponse {
        router: app,
        collection_response_vec: minter_collection_response,
        accts: Accounts { creator, buyer },
    })
}

pub fn base_minter_with_sudo_update_params_template(
    num_tokens: u32,
) -> MinterTemplateResponseCodeIds<Accounts> {
    let mut app = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut app);
    let start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    let collection_params = mock_collection_params_1(Some(start_time));
    let minter_params = minter_params_token(num_tokens);
    let code_ids = base_minter_sg721_collection_code_ids(&mut app);
    let minter_collection_response: Vec<MinterCollectionResponse> = configure_minter(
        &mut app,
        creator.clone(),
        vec![collection_params],
        vec![minter_params],
        code_ids.clone(),
    );
    MinterTemplateResponseCodeIds {
        router: app,
        collection_response_vec: minter_collection_response,
        accts: Accounts { creator, buyer },
        code_ids,
    }
}

pub fn vending_minter_template_with_code_ids_template(
    num_tokens: u32,
) -> MinterTemplateResponseCodeIds<Accounts> {
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
        code_ids.clone(),
    );
    MinterTemplateResponseCodeIds {
        router: app,
        collection_response_vec: minter_collection_response,
        accts: Accounts { creator, buyer },
        code_ids,
    }
}

pub fn open_edition_minter_with_two_sg721_collections_burn_mint(
    params_extension: ParamsExtension,
    init_msg: OpenEditionMinterInitMsgExtension,
) -> Result<MinterTemplateResponse<Accounts>, anyhow::Result<AppResponse>> {
    let mut router = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut router);
    router
        .sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: creator.to_string(),
                amount: vec![coin(CREATION_FEE * 2, NATIVE_DENOM)],
            }
        }))
        .map_err(|err| println!("{err:?}"))
        .ok();
    let start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    let collection_params = mock_collection_params_1(Some(start_time));
    let collection_params_2 = mock_collection_two(Some(start_time));
    let minter_params = minter_params_open_edition(
        params_extension.clone(),
        init_msg.clone(),
        None,
        None,
        None,
        None,
        None,
    );
    let minter_params_2 =
        minter_params_open_edition(params_extension, init_msg, None, None, None, None, None);
    let code_ids = open_edition_minter_code_ids(&mut router);
    let minter_collection_response = configure_open_edition_minter(
        &mut router,
        creator.clone(),
        vec![collection_params, collection_params_2],
        vec![minter_params, minter_params_2],
        code_ids,
    );
    Ok(MinterTemplateResponse {
        router,
        collection_response_vec: minter_collection_response,
        accts: Accounts { creator, buyer },
    })
}

pub fn open_edition_minter_ibc_template(
    params_extension: ParamsExtension,
    init_msg: OpenEditionMinterInitMsgExtension,
    custom_minter_params: OpenEditionMinterParams,
) -> Result<MinterTemplateResponseCodeIds<Accounts>, anyhow::Result<AppResponse>> {
    let mut app = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut app);
    let code_ids = open_edition_minter_code_ids(&mut app);
    let collection_params = mock_collection_params_1(None);
    let minter_params = minter_params_open_edition(
        params_extension,
        init_msg,
        None,
        None,
        None,
        None,
        Some(custom_minter_params),
    );

    let minter_collection_response = configure_open_edition_minter(
        &mut app,
        creator.clone(),
        vec![collection_params],
        vec![minter_params],
        code_ids.clone(),
    );
    Ok(MinterTemplateResponseCodeIds {
        router: app,
        collection_response_vec: minter_collection_response,
        accts: Accounts { creator, buyer },
        code_ids,
    })
}
