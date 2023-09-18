use cosmwasm_std::{Coin, Uint128};
use open_edition_factory::state::ParamsExtension;
use sg_std::NATIVE_DENOM;

use crate::common_setup::{
    contract_boxes::custom_mock_app,
    setup_minter::{
        common::constants::DEV_ADDRESS,
        open_edition_minter::{
            minter_params::{default_nft_data, init_msg},
            setup::open_edition_minter_code_ids,
        },
    },
    templates::open_edition_minter_custom_code_ids,
};

#[test]
fn invalid_code_id() {
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
    let mut app = custom_mock_app();
    let mut code_ids = open_edition_minter_code_ids(&mut app);
    code_ids.sg721_code_id = 19;
    let vt =
        open_edition_minter_custom_code_ids(app, params_extension, init_msg, code_ids).unwrap();
    assert_eq!(
        vt.collection_response_vec[0]
            .error
            .as_ref()
            .unwrap()
            .root_cause()
            .to_string(),
        "InvalidCollectionCodeId 19".to_string()
    );
}
