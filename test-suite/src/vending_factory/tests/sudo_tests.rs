use base_factory::msg::ParamsResponse;
use cosmwasm_std::{coin, Addr};
use sg_std::NATIVE_DENOM;
use vending_factory::msg::VendingUpdateParamsExtension;

use crate::common_setup::msg::MinterCollectionResponse;
use crate::common_setup::setup_minter::vending_minter::setup::sudo_update_params;
use crate::common_setup::templates::vending_minter_template_with_code_ids_template;
use sg2::query::Sg2QueryMsg::Params;

#[test]
fn happy_path_with_params_update() {
    let vt = vending_minter_template_with_code_ids_template(2);
    let (mut router, _, _) = (vt.router, vt.accts.creator, vt.accts.buyer);
    sudo_update_params(&mut router, &vt.collection_response_vec, vt.code_ids, None);
}

#[test]
fn sudo_params_update_invalid_nft_collection() {
    let vt = vending_minter_template_with_code_ids_template(2);
    let (mut router, _, _) = (vt.router, vt.accts.creator, vt.accts.buyer);
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
    let vt = vending_minter_template_with_code_ids_template(2);
    let (mut router, _, _) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let collection = vt.collection_response_vec[0].collection.clone().unwrap();
    let factory = vt.collection_response_vec[0].factory.clone().unwrap();
    let code_ids = vt.code_ids.clone();

    let update_msg = sg2::msg::UpdateMinterParamsMsg {
        code_id: Some(code_ids.sg721_code_id),
        add_sg721_code_ids: None,
        rm_sg721_code_ids: None,
        frozen: None,
        creation_fee: Some(coin(999, NATIVE_DENOM)),
        min_mint_price: Some(sg2::NonFungible(collection.to_string())),
        mint_fee_bps: None,
        max_trading_offset_secs: Some(100),
        extension: VendingUpdateParamsExtension {
            max_token_limit: None,
            max_per_address_limit: None,
            airdrop_mint_price: None,
            airdrop_mint_fee_bps: None,
            shuffle_fee: None,
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
