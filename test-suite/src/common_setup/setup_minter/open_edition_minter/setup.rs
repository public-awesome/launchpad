use crate::common_setup::contract_boxes::{
    contract_open_edition_factory, contract_open_edition_minter,
};
use crate::common_setup::msg::{
    MinterCollectionResponse, OpenEditionMinterInstantiateParams, OpenEditionMinterSetupParams,
};
use crate::common_setup::setup_minter::common::parse_response::build_collection_response;
use cosmwasm_std::{coin, coins, Addr, Coin, Timestamp};
use cw_multi_test::{Contract, Executor};
use open_edition_factory::msg::OpenEditionMinterInitMsgExtension;
use open_edition_factory::types::NftData;
use sg2::msg::{CollectionParams, Sg2ExecuteMsg};
use sg_multi_test::StargazeApp;
use sg_std::{StargazeMsgWrapper, NATIVE_DENOM};

use crate::common_setup::msg::CodeIds;
use crate::common_setup::setup_minter::open_edition_minter::mock_params::{
    mock_create_minter, mock_init_minter_extension, mock_params_proper,
};

use crate::common_setup::setup_minter::common::constants::CREATION_FEE;

pub fn build_init_msg(
    init_msg: Option<OpenEditionMinterInitMsgExtension>,
    start_time: Option<Timestamp>,
    end_time: Option<Timestamp>,
    per_address_limit_minter: Option<u32>,
    nft_data: NftData,
    mint_price: Option<Coin>,
    payment_address: Option<String>,
) -> OpenEditionMinterInitMsgExtension {
    match init_msg {
        Some(init_msg_from_params) => init_msg_from_params,
        None => mock_init_minter_extension(
            start_time,
            end_time,
            per_address_limit_minter,
            mint_price,
            nft_data,
            payment_address,
        ),
    }
}

// Upload contract code and instantiate open edition minter contract
pub fn setup_open_edition_minter_contract(
    setup_params: OpenEditionMinterSetupParams,
) -> MinterCollectionResponse {
    let minter_code_id = setup_params.minter_code_id;
    let router = setup_params.router;
    let factory_code_id = setup_params.factory_code_id;
    let sg721_code_id = setup_params.sg721_code_id;
    let minter_admin = setup_params.minter_admin;
    let collection_params = setup_params.collection_params;
    let start_time = setup_params.start_time;
    let end_time = setup_params.end_time;
    let init_msg = setup_params.init_msg.clone();
    let nft_data = setup_params.init_msg.unwrap().nft_data;

    let mut params = mock_params_proper();
    params.code_id = minter_code_id;

    let factory_addr = router.instantiate_contract(
        factory_code_id,
        minter_admin.clone(),
        &open_edition_factory::msg::InstantiateMsg {
            params: params.clone(),
        },
        &[],
        "factory",
        None,
    );

    let factory_addr = factory_addr.unwrap();
    let min_mint_price = params.min_mint_price.clone().fungible_coin().unwrap();
    let denom = params.min_mint_price.denom().unwrap();
    let mut msg = mock_create_minter(
        start_time,
        end_time,
        Some(coin(min_mint_price.u128(), denom.clone())),
        Some(params.extension.max_per_address_limit),
        nft_data.clone(),
        collection_params,
        None,
    );
    msg.init_msg = build_init_msg(
        init_msg,
        start_time,
        end_time,
        Some(params.extension.max_per_address_limit),
        nft_data,
        Some(coin(min_mint_price.u128(), denom)),
        None,
    );
    msg.collection_params.code_id = sg721_code_id;
    msg.collection_params.info.creator = minter_admin.to_string();
    let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);
    let msg = Sg2ExecuteMsg::CreateMinter(msg);

    let res = router.execute_contract(minter_admin, factory_addr.clone(), &msg, &creation_fee);
    build_collection_response(res, factory_addr)
}

pub fn open_edition_minter_code_ids(
    router: &mut StargazeApp,
    sg721_code: Box<dyn Contract<StargazeMsgWrapper>>,
) -> CodeIds {
    let minter_code_id = router.store_code(contract_open_edition_minter());

    let factory_code_id = router.store_code(contract_open_edition_factory());

    let sg721_code_id = router.store_code(sg721_code);

    CodeIds {
        minter_code_id,
        factory_code_id,
        sg721_code_id,
    }
}

pub fn configure_open_edition_minter(
    app: &mut StargazeApp,
    minter_admin: Addr,
    collection_params_vec: Vec<CollectionParams>,
    minter_instantiate_params_vec: Vec<OpenEditionMinterInstantiateParams>,
    code_ids: CodeIds,
) -> Vec<MinterCollectionResponse> {
    let mut minter_collection_info: Vec<MinterCollectionResponse> = vec![];
    for (index, collection_param) in collection_params_vec.iter().enumerate() {
        let setup_params: OpenEditionMinterSetupParams = OpenEditionMinterSetupParams {
            router: app,
            minter_admin: minter_admin.clone(),
            collection_params: collection_param.to_owned(),
            minter_code_id: code_ids.minter_code_id,
            factory_code_id: code_ids.factory_code_id,
            sg721_code_id: code_ids.sg721_code_id,
            start_time: minter_instantiate_params_vec[index].start_time.to_owned(),
            nft_data: minter_instantiate_params_vec[index]
                .nft_data
                .to_owned()
                .unwrap(),
            per_address_limit: minter_instantiate_params_vec[index]
                .per_address_limit
                .to_owned()
                .unwrap(),
            init_msg: minter_instantiate_params_vec[index].init_msg.clone(),
            end_time: minter_instantiate_params_vec[index].end_time.to_owned(),
        };
        let minter_collection_res = setup_open_edition_minter_contract(setup_params);
        minter_collection_info.push(minter_collection_res);
    }
    minter_collection_info
}
