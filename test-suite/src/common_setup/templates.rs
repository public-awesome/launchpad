use crate::common_setup::contract_boxes::{contract_sg721_base, custom_mock_app};
use crate::common_setup::msg::MinterTemplateResponse;
use crate::common_setup::{
    msg::MinterCollectionResponse,
    setup_accounts_and_block::setup_accounts,
    setup_minter::common::minter_params::minter_params_token,
    setup_minter::vending_minter::setup::{configure_minter, vending_minter_code_ids},
};

use super::msg::{Accounts, MinterTemplateResponseCodeIds};
use super::setup_minter::base_minter::setup::base_minter_sg721_collection_code_ids;
use super::setup_minter::common::constants::{MINT_PRICE, MIN_MINT_PRICE};
use super::setup_minter::common::minter_params::minter_params_all;
use super::setup_minter::open_edition_minter::setup::configure_open_edition_minter;
// use super::setup_minter::open_edition_minter::setup::configure_open_edition_minter;
use crate::common_setup::setup_minter::base_minter::setup::base_minter_sg721_nt_code_ids;
use crate::common_setup::setup_minter::base_minter::setup::configure_base_minter;
use crate::common_setup::setup_minter::common::constants::{
    CREATION_FEE, MIN_MINT_PRICE_OPEN_EDITION,
};
use crate::common_setup::setup_minter::common::parse_response::build_collection_response;
use crate::common_setup::setup_minter::open_edition_minter::mock_params::{
    mock_create_minter, mock_init_minter_extension, mock_params_proper,
};
use crate::common_setup::setup_minter::open_edition_minter::setup::open_edition_minter_code_ids;
use crate::common_setup::setup_minter::vending_minter::setup::vending_minter_updatable_code_ids;
use cosmwasm_std::{coin, coins, Coin, Timestamp, Uint128};
use cw_multi_test::{AppResponse, Contract, Executor};
use open_edition_factory::msg::OpenEditionMinterCreateMsg;
use open_edition_factory::state::ParamsExtension;
use open_edition_factory::types::{NftData, NftMetadataType};
use sg2::msg::Sg2ExecuteMsg;
use sg2::tests::mock_collection_params_1;
use sg_multi_test::StargazeApp;
use sg_std::{StargazeMsgWrapper, GENESIS_MINT_START_TIME, NATIVE_DENOM};

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
    mut app: StargazeApp,
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

// #[allow(clippy::too_many_arguments)]
// pub fn open_edition_minter_custom_template(
//     start_minter_time: Option<Timestamp>,
//     end_minter_time: Option<Timestamp>,
//     nft_data: Option<NftData>,
//     max_per_address_limit: Option<u32>,
//     per_address_limit_minter: Option<u32>,
//     mint_price_minter: Option<Coin>,
//     sg721_code: Option<Box<dyn Contract<StargazeMsgWrapper>>>,
//     sg721_codeid: Option<u64>,
// ) -> Result<MinterTemplateResponse<Accounts>, anyhow::Result<AppResponse>> {

// Custom params set to a high level function to ease the tests
#[allow(clippy::too_many_arguments)]
pub fn open_edition_minter_custom_template(
    max_per_address_limit: Option<u32>,
    per_address_limit_minter: Option<u32>,
) -> Result<MinterTemplateResponse<Accounts>, anyhow::Result<AppResponse>> {
    let mut app = custom_mock_app();
    let (creator, buyer) = setup_accounts(&mut app);
    let code_ids = open_edition_minter_code_ids(&mut app, contract_sg721_base());

    use crate::common_setup::setup_minter::open_edition_minter::minter_params::minter_params_open_edition;
    let collection_params = mock_collection_params_1(None);
    // test-suite/src/common_setup/setup_minter/open_edition_minter/minter_params.rs

    use crate::common_setup::setup_minter::common::constants::DEV_ADDRESS;

    let mut params_extension = ParamsExtension {
        max_per_address_limit: 10,
        airdrop_mint_fee_bps: 100,
        airdrop_mint_price: Coin {
            denom: NATIVE_DENOM.to_string(),
            amount: Uint128::new(100_000_000u128),
        },
        dev_fee_address: DEV_ADDRESS.to_string(),
    };
    if max_per_address_limit.is_some() {
        params_extension.max_per_address_limit = max_per_address_limit.unwrap();
    }

    let default_nft_data = NftData {
        nft_data_type: NftMetadataType::OffChainMetadata,
        extension: None,
        token_uri: Some(
            "ipfs://bafybeiavall5udkxkdtdm4djezoxrmfc6o5fn2ug3ymrlvibvwmwydgrkm/1.jpg".to_string(),
        ),
    };

    let init_msg = mock_init_minter_extension(
        None,
        None,
        per_address_limit_minter,
        Some(Coin {
            denom: NATIVE_DENOM.to_string(),
            amount: Uint128::new(MIN_MINT_PRICE_OPEN_EDITION),
        }),
        default_nft_data,
        None,
    );

    let minter_params =
        minter_params_open_edition(params_extension, per_address_limit_minter, Some(init_msg));

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

// pub fn open_edition_minter_with_sudo_update_params_template(
//     num_tokens: u32,
// ) -> MinterTemplateResponseCodeIds<Accounts> {
//     let mut app = custom_mock_app();
//     let (creator, buyer) = setup_accounts(&mut app);
//     let start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
//     let collection_params = mock_collection_params_1(Some(start_time));
//     let minter_params = minter_params_token(num_tokens);
//     let code_ids = open_edition_minter_sg721_collection_code_ids(&mut app);
//     let minter_collection_response: Vec<MinterCollectionResponse> = configure_open_edition_minter(
//         &mut app,
//         creator.clone(),
//         vec![collection_params],
//         vec![minter_params],
//         code_ids.clone(),
//     );
//     MinterTemplateResponseCodeIds {
//         router: app,
//         collection_response_vec: minter_collection_response,
//         accts: Accounts { creator, buyer },
//         code_ids,
//     }
// }
