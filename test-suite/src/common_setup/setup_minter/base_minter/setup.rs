use crate::common_setup::contract_boxes::contract_base_minter;
use crate::common_setup::contract_boxes::contract_nt_collection;
use crate::common_setup::contract_boxes::contract_sg721_base;
use crate::common_setup::contract_boxes::{contract_base_factory, App};
use crate::common_setup::msg::MinterCollectionResponse;
use crate::common_setup::msg::MinterSetupParams;
use crate::common_setup::setup_minter::common::constants::MIN_MINT_PRICE;
use crate::common_setup::setup_minter::common::parse_response::build_collection_response;
use cosmwasm_std::coin;
use cosmwasm_std::to_json_binary;
use cosmwasm_std::Coin;
use cosmwasm_std::Empty;
use cosmwasm_std::{coins, Addr};
use cw_multi_test::AppResponse;
use cw_multi_test::Executor;
use sg2::msg::{CollectionParams, Sg2ExecuteMsg};
use sg_std::NATIVE_DENOM;

use crate::common_setup::msg::{CodeIds, MinterInstantiateParams};
use crate::common_setup::setup_minter::base_minter::mock_params::{
    mock_create_minter, mock_params,
};
use crate::common_setup::setup_minter::common::constants::CREATION_FEE;

pub fn base_minter_sg721_nt_code_ids(router: &mut App) -> CodeIds {
    let minter_code_id = router.store_code(contract_base_minter());
    println!("base_minter_code_id: {minter_code_id}");

    let factory_code_id = router.store_code(contract_base_factory());
    println!("base_factory_code_id: {factory_code_id}");

    let sg721_code_id = router.store_code(contract_nt_collection());
    println!("sg721nt_code_id: {sg721_code_id}");
    CodeIds {
        minter_code_id,
        factory_code_id,
        sg721_code_id,
    }
}

pub fn base_minter_sg721_collection_code_ids(router: &mut App) -> CodeIds {
    let minter_code_id = router.store_code(contract_base_minter());
    println!("base_minter_code_id: {minter_code_id}");

    let factory_code_id = router.store_code(contract_base_factory());
    println!("base_factory_code_id: {factory_code_id}");

    let sg721_code_id = router.store_code(contract_sg721_base());
    println!("sg721_code_id: {sg721_code_id}");
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

pub fn sudo_update_params(
    app: &mut App,
    collection_responses: &Vec<MinterCollectionResponse>,
    code_ids: CodeIds,
    update_msg: Option<sg2::msg::UpdateMinterParamsMsg<Empty>>,
) -> Vec<Result<AppResponse, anyhow::Error>> {
    let mut sudo_responses: Vec<Result<AppResponse, anyhow::Error>> = vec![];
    for collection_response in collection_responses {
        let update_msg = match update_msg.clone() {
            Some(some_update_message) => some_update_message,
            None => sg2::msg::UpdateMinterParamsMsg {
                code_id: Some(code_ids.sg721_code_id),
                add_sg721_code_ids: None,
                rm_sg721_code_ids: None,
                frozen: None,
                creation_fee: Some(coin(0, NATIVE_DENOM)),
                min_mint_price: Some(Coin {
                    amount: MIN_MINT_PRICE.into(),
                    denom: NATIVE_DENOM.into(),
                }),
                mint_fee_bps: None,
                max_trading_offset_secs: Some(100),
                extension: Empty {},
            },
        };
        let sudo_update_msg = base_factory::msg::SudoMsg::UpdateParams(Box::new(update_msg));

        let sudo_res = app.sudo(cw_multi_test::SudoMsg::Wasm(cw_multi_test::WasmSudo {
            contract_addr: collection_response.factory.clone().unwrap(),
            msg: to_json_binary(&sudo_update_msg).unwrap(),
        }));
        sudo_responses.push(sudo_res);
    }
    sudo_responses
}

pub fn configure_base_minter(
    app: &mut App,
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
