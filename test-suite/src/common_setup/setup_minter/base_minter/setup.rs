use crate::common_setup::contract_boxes::contract_base_factory;
use crate::common_setup::contract_boxes::contract_base_minter;
use crate::common_setup::contract_boxes::contract_nt_collection;
use crate::common_setup::contract_boxes::contract_sg721;
use crate::common_setup::msg::MinterCollectionResponse;
use crate::common_setup::msg::MinterSetupParams;
use crate::common_setup::setup_minter::common::parse_response::build_collection_response;
use cosmwasm_std::{coins, Addr};
use cw_multi_test::Executor;
use sg2::msg::{CollectionParams, Sg2ExecuteMsg};
use sg_multi_test::StargazeApp;
use sg_std::NATIVE_DENOM;

use crate::common_setup::msg::{CodeIds, MinterInstantiateParams};
use crate::common_setup::setup_minter::base_minter::mock_params::{
    mock_create_minter, mock_params,
};
use crate::common_setup::setup_minter::common::constants::CREATION_FEE;

pub fn base_minter_sg721_nt_code_ids(router: &mut StargazeApp) -> CodeIds {
    let minter_code_id = router.store_code(contract_base_minter());
    println!("base_minter_code_id: {}", minter_code_id);

    let factory_code_id = router.store_code(contract_base_factory());
    println!("base_factory_code_id: {}", factory_code_id);

    let sg721_code_id = router.store_code(contract_nt_collection());
    println!("sg721nt_code_id: {}", sg721_code_id);
    CodeIds {
        minter_code_id,
        factory_code_id,
        sg721_code_id,
    }
}

pub fn base_minter_sg721_collection_code_ids(router: &mut StargazeApp) -> CodeIds {
    let minter_code_id = router.store_code(contract_base_minter());
    println!("base_minter_code_id: {}", minter_code_id);

    let factory_code_id = router.store_code(contract_base_factory());
    println!("base_factory_code_id: {}", factory_code_id);

    let sg721_code_id = router.store_code(contract_sg721());
    println!("sg721_code_id: {}", sg721_code_id);
    CodeIds {
        minter_code_id,
        factory_code_id,
        sg721_code_id,
    }
}

// Upload contract code and instantiate minter contract
pub fn setup_minter_contract(setup_params: MinterSetupParams) -> MinterCollectionResponse {
    let minter_code_id = setup_params.minter_code_id;
    let router = setup_params.router;
    let factory_code_id = setup_params.factory_code_id;
    let sg721_code_id = setup_params.sg721_code_id;
    let minter_admin = setup_params.minter_admin;
    let collection_params = setup_params.collection_params;

    let mut params = mock_params();
    params.code_id = minter_code_id;

    let factory_addr = router
        .instantiate_contract(
            factory_code_id,
            minter_admin.clone(),
            &base_factory::msg::InstantiateMsg { params },
            &[],
            "factory",
            None,
        )
        .unwrap();

    let mut msg = mock_create_minter(collection_params);
    msg.collection_params.code_id = sg721_code_id;
    msg.collection_params.info.creator = minter_admin.to_string();
    let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);
    let msg = Sg2ExecuteMsg::CreateMinter(msg);

    let res = router.execute_contract(minter_admin, factory_addr.clone(), &msg, &creation_fee);
    build_collection_response(res, factory_addr)
}

pub fn configure_base_minter(
    app: &mut StargazeApp,
    minter_admin: Addr,
    collection_params_vec: Vec<CollectionParams>,
    minter_instantiate_params_vec: Vec<MinterInstantiateParams>,
    code_ids: CodeIds,
) -> Vec<MinterCollectionResponse> {
    let mut minter_collection_info: Vec<MinterCollectionResponse> = vec![];
    for (index, collection_param) in collection_params_vec.iter().enumerate() {
        let setup_params: MinterSetupParams = MinterSetupParams {
            router: app,
            minter_admin: minter_admin.clone(),
            num_tokens: minter_instantiate_params_vec[index].num_tokens,
            collection_params: collection_param.to_owned(),
            splits_addr: minter_instantiate_params_vec[index].splits_addr.clone(),
            minter_code_id: code_ids.minter_code_id,
            factory_code_id: code_ids.factory_code_id,
            sg721_code_id: code_ids.sg721_code_id,
            start_time: minter_instantiate_params_vec[index].start_time,
            init_msg: minter_instantiate_params_vec[index].init_msg.clone(),
        };
        let minter_collection_res = setup_minter_contract(setup_params);
        minter_collection_info.push(minter_collection_res);
    }
    minter_collection_info
}
