use crate::common_setup::contract_boxes::{
    contract_sg721_base, contract_vending_factory, contract_vending_minter,
};
use crate::common_setup::msg::MinterCollectionResponse;
use crate::common_setup::msg::MinterSetupParams;
use crate::common_setup::setup_minter::common::parse_response::build_collection_response;
use cosmwasm_std::{coin, coins, Addr};
use cw_multi_test::Executor;
use sg2::msg::CreateMinterMsg;
use sg2::msg::{CollectionParams, Sg2ExecuteMsg};
use sg_multi_test::StargazeApp;
use sg_std::NATIVE_DENOM;
use vending_factory::msg::VendingMinterInitMsgExtension;

use crate::common_setup::msg::{CodeIds, MinterInstantiateParams};
use crate::common_setup::setup_minter::vending_minter::mock_params::{
    mock_create_minter, mock_params,
};

use crate::common_setup::setup_minter::common::constants::{CREATION_FEE, MINT_PRICE};

pub fn build_init_msg(
    init_msg: Option<VendingMinterInitMsgExtension>,
    mut msg: CreateMinterMsg<VendingMinterInitMsgExtension>,
    num_tokens: u32,
) -> VendingMinterInitMsgExtension {
    match init_msg {
        Some(init_msg_from_params) => init_msg_from_params,
        None => {
            msg.init_msg.mint_price = coin(MINT_PRICE, NATIVE_DENOM);
            msg.init_msg.num_tokens = num_tokens;
            msg.init_msg
        }
    }
}

// Upload contract code and instantiate minter contract
pub fn setup_minter_contract(setup_params: MinterSetupParams) -> MinterCollectionResponse {
    let minter_code_id = setup_params.minter_code_id;
    let router = setup_params.router;
    let factory_code_id = setup_params.factory_code_id;
    let sg721_code_id = setup_params.sg721_code_id;
    let minter_admin = setup_params.minter_admin;
    let num_tokens = setup_params.num_tokens;
    let splits_addr = setup_params.splits_addr;
    let collection_params = setup_params.collection_params;
    let start_time = setup_params.start_time;
    let init_msg = setup_params.init_msg;

    let mint_denom: Option<String> = init_msg
        .as_ref()
        .map(|msg| msg.mint_price.denom.to_string());

    let mut params = mock_params(mint_denom);
    params.code_id = minter_code_id;

    let factory_addr = router
        .instantiate_contract(
            factory_code_id,
            minter_admin.clone(),
            &vending_factory::msg::InstantiateMsg { params },
            &[],
            "factory",
            None,
        )
        .unwrap();
    let mut msg = mock_create_minter(splits_addr, collection_params, start_time);
    msg.init_msg = build_init_msg(init_msg, msg.clone(), num_tokens);
    msg.collection_params.code_id = sg721_code_id;
    msg.collection_params.info.creator = minter_admin.to_string();
    let creation_fee = coins(CREATION_FEE, NATIVE_DENOM);
    let msg = Sg2ExecuteMsg::CreateMinter(msg);

    let res = router.execute_contract(minter_admin, factory_addr.clone(), &msg, &creation_fee);
    build_collection_response(res, factory_addr)
}

pub fn vending_minter_code_ids(router: &mut StargazeApp) -> CodeIds {
    let minter_code_id = router.store_code(contract_vending_minter());
    println!("minter_code_id: {}", minter_code_id);

    let factory_code_id = router.store_code(contract_vending_factory());
    println!("factory_code_id: {}", factory_code_id);

    let sg721_code_id = router.store_code(contract_sg721_base());
    println!("sg721_code_id: {}", sg721_code_id);
    CodeIds {
        minter_code_id,
        factory_code_id,
        sg721_code_id,
    }
}

pub fn configure_minter(
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
