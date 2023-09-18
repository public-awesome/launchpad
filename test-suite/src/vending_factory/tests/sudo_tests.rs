use base_factory::msg::ParamsResponse;
use cosmwasm_std::coin;
use sg_std::NATIVE_DENOM;
use vending_factory::msg::VendingUpdateParamsExtension;

use crate::common_setup::setup_minter::base_minter::mock_params::MIN_MINT_PRICE;
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
fn sudo_params_update_creation_fee() {
    let vt = vending_minter_template_with_code_ids_template(2);
    let (mut router, _, _) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let factory = vt.collection_response_vec[0].factory.clone().unwrap();
    let code_ids = vt.code_ids.clone();

    let update_msg = sg2::msg::UpdateMinterParamsMsg {
        code_id: Some(code_ids.sg721_code_id),
        add_sg721_code_ids: None,
        rm_sg721_code_ids: None,
        frozen: None,
        creation_fee: Some(coin(999, NATIVE_DENOM)),
        min_mint_price: Some(coin(MIN_MINT_PRICE, NATIVE_DENOM)),
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
