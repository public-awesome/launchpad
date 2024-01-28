use cosmwasm_std::{coins, Coin, Timestamp, Uint128};
use cw_multi_test::{BankSudo, Executor, SudoMsg};
use open_edition_factory::state::ParamsExtension;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};

use open_edition_factory::msg::{OpenEditionUpdateParamsExtension, OpenEditionUpdateParamsMsg};
use open_edition_factory::types::{NftData, NftMetadataType};
use sg2::msg::Sg2ExecuteMsg;
use sg2::tests::mock_collection_params_1;

use crate::common_setup::setup_minter::common::constants::MIN_MINT_PRICE_OPEN_EDITION;
use crate::common_setup::setup_minter::common::constants::{CREATION_FEE, DEV_ADDRESS};
use crate::common_setup::setup_minter::open_edition_minter::minter_params::{
    default_nft_data, init_msg,
};
use crate::common_setup::setup_minter::open_edition_minter::mock_params::mock_create_minter;
use crate::common_setup::templates::open_edition_minter_custom_template;

#[test]
fn frozen_factory_cannot_create_new_minters() {
    let params_extension = ParamsExtension {
        max_token_limit: 10,
        max_per_address_limit: 10,
        airdrop_mint_fee_bps: 100,
        airdrop_mint_price: Coin {
            denom: NATIVE_DENOM.to_string(),
            amount: Uint128::new(100_000_000u128),
        },
        dev_fee_address: DEV_ADDRESS.to_string(),
    };
    let init_msg = init_msg(
        default_nft_data(),
        None,
        None,
        Some(Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10_000)),
        None,
        None,
    );
    let vt = open_edition_minter_custom_template(params_extension, init_msg).unwrap();

    let (mut router, creator, _buyer) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let _minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();
    let factory_addr = vt.collection_response_vec[0].factory.clone().unwrap();

    let update_msg = OpenEditionUpdateParamsMsg {
        add_sg721_code_ids: None,
        rm_sg721_code_ids: None,
        frozen: Some(true),
        code_id: None,
        creation_fee: None,
        min_mint_price: None,
        mint_fee_bps: None,
        max_trading_offset_secs: None,
        extension: OpenEditionUpdateParamsExtension {
            max_token_limit: None,
            max_per_address_limit: None,
            min_mint_price: None,
            airdrop_mint_fee_bps: None,
            airdrop_mint_price: None,
            dev_fee_address: None,
        },
    };

    let sudo_msg = open_edition_factory::msg::SudoMsg::UpdateParams(Box::new(update_msg));
    let _res = router.wasm_sudo(factory_addr.clone(), &sudo_msg);

    // Creating a new one should error
    let start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 100);
    let end_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME + 10_000);
    let per_address_limit_minter = Some(1);
    let mint_price = Some(Coin {
        denom: NATIVE_DENOM.to_string(),
        amount: Uint128::new(MIN_MINT_PRICE_OPEN_EDITION),
    });
    let collection_params = mock_collection_params_1(Some(start_time));
    let default_nft_data = NftData {
        nft_data_type: NftMetadataType::OffChainMetadata,
        extension: None,
        token_uri: Some(
            "ipfs://bafybeiavall5udkxkdtdm4djezoxrmfc6o5fn2ug3ymrlvibvwmwydgrkm/1.jpg".to_string(),
        ),
    };
    let mut msg = mock_create_minter(
        Some(start_time),
        Some(end_time),
        mint_price,
        per_address_limit_minter,
        None,
        default_nft_data,
        collection_params,
        None,
    );
    msg.collection_params.code_id = 3;
    msg.collection_params.info.creator = creator.to_string();
    let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);
    let msg = Sg2ExecuteMsg::CreateMinter(msg);
    router
        .sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: creator.to_string(),
                amount: coins(CREATION_FEE, NATIVE_DENOM),
            }
        }))
        .map_err(|err| println!("{err:?}"))
        .ok();
    let res = router.execute_contract(creator, factory_addr, &msg, &creation_fee);
    assert_eq!(
        res.err().unwrap().source().unwrap().to_string(),
        "Factory frozen. Cannot make new minters."
    );
}
