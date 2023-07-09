use crate::common_setup::msg::MinterCollectionResponse;
use crate::common_setup::setup_minter::common::constants::DEV_ADDRESS;
use crate::common_setup::setup_minter::open_edition_minter::minter_params::{
    default_nft_data, init_msg,
};
use crate::common_setup::setup_minter::open_edition_minter::setup::sudo_update_params;
use crate::common_setup::templates::open_edition_minter_custom_template;
use base_factory::msg::ParamsResponse;
use cosmwasm_std::{coin, Addr, Coin, Uint128};
use open_edition_factory::msg::OpenEditionUpdateParamsExtension;
use open_edition_factory::state::ParamsExtension;
use sg2::query::Sg2QueryMsg::Params;
use sg_std::NATIVE_DENOM;

#[test]
fn happy_path_with_params_update() {
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
    let init_msg = init_msg(
        default_nft_data(),
        per_address_limit_minter,
        None,
        None,
        None,
    );
    let vt = open_edition_minter_custom_template(params_extension, init_msg).unwrap();
    let mut router = vt.router;
    let res = sudo_update_params(&mut router, &vt.collection_response_vec, vt.code_ids, None);
    println!("res is {:?}", res);
}

#[test]
fn sudo_params_update_invalid_nft_collection() {
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
    let init_msg = init_msg(
        default_nft_data(),
        per_address_limit_minter,
        None,
        None,
        None,
    );
    let vt = open_edition_minter_custom_template(params_extension, init_msg).unwrap();
    let mut router = vt.router;
    let mut response_collection_vec: Vec<MinterCollectionResponse> = vec![];
    for entry in vt.collection_response_vec {
        let new_entry = MinterCollectionResponse {
            minter: entry.minter,
            collection: Some(Addr::unchecked("fake_address")),
            factory: entry.factory,
            error: entry.error,
        };
        response_collection_vec.push(new_entry);
    }
    let sudo_responses =
        sudo_update_params(&mut router, &response_collection_vec, vt.code_ids, None);
    let sudo_response_1 = sudo_responses.first();
    let err = sudo_response_1.unwrap().as_ref().unwrap_err().to_string();
    assert_eq!(err, "InvalidCollectionAddress".to_string());
}

#[test]
fn sudo_params_update_creation_fee() {
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
    let init_msg = init_msg(
        default_nft_data(),
        per_address_limit_minter,
        None,
        None,
        None,
    );
    let vt = open_edition_minter_custom_template(params_extension, init_msg).unwrap();
    let collection = vt.collection_response_vec[0].collection.clone().unwrap();
    let factory = vt.collection_response_vec[0].factory.clone().unwrap();
    let code_ids = vt.code_ids.clone();
    let mut router = vt.router;

    let update_msg = sg2::msg::UpdateMinterParamsMsg {
        code_id: Some(code_ids.sg721_code_id),
        add_sg721_code_ids: None,
        rm_sg721_code_ids: None,
        frozen: None,
        creation_fee: Some(coin(999, NATIVE_DENOM)),
        min_mint_price: Some(sg2::NonFungible(collection.to_string())),
        mint_fee_bps: None,
        max_trading_offset_secs: Some(100),
        extension: OpenEditionUpdateParamsExtension {
            min_mint_price: Some(coin(10, NATIVE_DENOM)),
            max_per_address_limit: None,
            airdrop_mint_price: None,
            airdrop_mint_fee_bps: None,
            dev_fee_address: Some(DEV_ADDRESS.to_string()),
        },
    };
    sudo_update_params(
        &mut router,
        &vt.collection_response_vec,
        vt.code_ids,
        Some(update_msg),
    );

    let res: ParamsResponse = router.wrap().query_wasm_smart(factory, &Params {}).unwrap();
    assert_eq!(res.params.creation_fee, coin(999, NATIVE_DENOM));
}
