use cosmwasm_std::{coins, Coin, Uint128};
use cw_multi_test::Executor;
use open_edition_factory::state::ParamsExtension;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};

use open_edition_minter::msg::ConfigResponse;
use open_edition_minter::msg::{ExecuteMsg, QueryMsg};

use crate::common_setup::setup_accounts_and_block::setup_block_time;
use crate::common_setup::setup_minter::common::constants::DEV_ADDRESS;
use crate::common_setup::setup_minter::open_edition_minter::minter_params::{
    default_nft_data, init_msg,
};
use crate::common_setup::templates::open_edition_minter_custom_template;

const MINT_PRICE: u128 = 100_000_000;

// // Custom params set to a high level function to ease the tests
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
//     let mut app = custom_mock_app();
//     let (creator, buyer) = setup_accounts(&mut app);
//     let code_ids =
//         open_edition_minter_code_ids(&mut app, sg721_code.unwrap_or_else(contract_sg721_base));

//     // Factory params
//     let mut factory_params = mock_params_proper();
//     factory_params.extension.max_per_address_limit =
//         max_per_address_limit.unwrap_or(factory_params.extension.max_per_address_limit);

//     let factory_addr = app.instantiate_contract(
//         code_ids.factory_code_id,
//         creator.clone(),
//         &open_edition_factory::msg::InstantiateMsg {
//             params: factory_params,
//         },
//         &[],
//         "factory",
//         None,
//     );

//     let factory_addr = factory_addr.unwrap();

//     // Minter -> Default params
//     let start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 100);
//     let end_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10_000);
//     let per_address_limit_minter = per_address_limit_minter.or(Some(1));
//     let mint_price = mint_price_minter.or_else(|| {
//         Some(Coin {
//             denom: NATIVE_DENOM.to_string(),
//             amount: Uint128::new(MIN_MINT_PRICE_OPEN_EDITION),
//         })
//     });
//     let collection_params = mock_collection_params_1(Some(start_time));
//     let default_nft_data = nft_data.unwrap_or(NftData {
//         nft_data_type: NftMetadataType::OffChainMetadata,
//         extension: None,
//         token_uri: Some(
//             "ipfs://bafybeiavall5udkxkdtdm4djezoxrmfc6o5fn2ug3ymrlvibvwmwydgrkm/1.jpg".to_string(),
//         ),
//     });
//     let mut msg = mock_create_minter(
//         start_minter_time.or(Some(start_time)),
//         end_minter_time.or(Some(end_time)),
//         mint_price,
//         per_address_limit_minter,
//         default_nft_data,
//         collection_params,
//         None,
//     );
//     msg.collection_params.code_id = sg721_codeid.unwrap_or(3);
//     msg.collection_params.info.creator = creator.to_string();
//     let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);
//     let msg = Sg2ExecuteMsg::CreateMinter(msg);

//     let res = app.execute_contract(creator.clone(), factory_addr.clone(), &msg, &creation_fee);
//     if res.is_err() {
//         return Err(res);
//     }

//     let minter_collection_res = build_collection_response(res, factory_addr);

//     Ok(MinterTemplateResponse {
//         router: app,
//         collection_response_vec: vec![minter_collection_res],
//         accts: Accounts { creator, buyer },
//     })
// }

#[test]
fn check_per_address_limit() {
    let params_extension = ParamsExtension {
        max_per_address_limit: 10,
        airdrop_mint_fee_bps: 100,
        airdrop_mint_price: Coin {
            denom: NATIVE_DENOM.to_string(),
            amount: Uint128::new(100_000_000u128),
        },
        dev_fee_address: DEV_ADDRESS.to_string(),
    };
    let per_address_limit_minter = Some(2);
    let init_msg = init_msg(default_nft_data(), per_address_limit_minter);
    let vt = open_edition_minter_custom_template(params_extension, init_msg).unwrap();

    let (mut router, creator, buyer) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();
    // Set to a valid mint time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 101, None);

    // Check the Config
    let query_config_msg = QueryMsg::Config {};
    let res: ConfigResponse = router
        .wrap()
        .query_wasm_smart(minter_addr.clone(), &query_config_msg)
        .unwrap();
    assert_eq!(res.per_address_limit, 2);

    // Set a new limit per address, check unauthorized
    let per_address_limit_msg = ExecuteMsg::UpdatePerAddressLimit {
        per_address_limit: 5,
    };
    let res = router.execute_contract(
        buyer.clone(), // unauthorized
        minter_addr.clone(),
        &per_address_limit_msg,
        &[],
    );
    assert_eq!(
        res.err().unwrap().source().unwrap().to_string(),
        "Unauthorized: Sender is not an admin"
    );

    // Set limit errors, invalid limit over max
    // Factory is set to 10 in the current case
    let per_address_limit_msg = ExecuteMsg::UpdatePerAddressLimit {
        per_address_limit: 30,
    };
    let res = router.execute_contract(
        creator.clone(),
        minter_addr.clone(),
        &per_address_limit_msg,
        &[],
    );
    assert_eq!(
        res.err().unwrap().source().unwrap().to_string(),
        "Invalid minting limit per address. max: 10, min: 1, got: 30"
    );

    // Set limit errors, invalid limit == 0
    let per_address_limit_msg = ExecuteMsg::UpdatePerAddressLimit {
        per_address_limit: 0,
    };
    let res = router.execute_contract(creator, minter_addr.clone(), &per_address_limit_msg, &[]);
    assert_eq!(
        res.err().unwrap().source().unwrap().to_string(),
        "Invalid minting limit per address. max: 10, min: 1, got: 0"
    );

    // Only the first 2 mints
    for _ in 1..=2 {
        let mint_msg = ExecuteMsg::Mint {};
        let res = router.execute_contract(
            buyer.clone(),
            minter_addr.clone(),
            &mint_msg,
            &coins(MINT_PRICE, NATIVE_DENOM),
        );
        assert!(res.is_ok());
    }

    // 3rd mint fails from exceeding per address limit
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer,
        minter_addr,
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert_eq!(
        res.err().unwrap().source().unwrap().to_string(),
        "Max minting limit per address exceeded"
    );
}
