use base_factory::msg::ParamsResponse;
use cosmwasm_std::coin;
use sg_utils::NATIVE_DENOM;
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

#[test]
fn test_factory_whitelist_management() {
    use vending_factory::msg::{IsContractWhitelistedResponse, QueryMsg as FactoryQueryMsg, SudoMsg, WhitelistedContractsResponse};

    let vt = vending_minter_template_with_code_ids_template(1);
    let (mut router, _, _) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let factory = vt.collection_response_vec[0].factory.clone().unwrap();

    let contract_addr = "contract1234567890abcdefghijklmnopqrstuvwxyz1234567890abcdefghij";

    // Initially, contract should not be whitelisted
    let query_msg = FactoryQueryMsg::IsContractWhitelisted {
        address: contract_addr.to_string(),
    };
    let res: IsContractWhitelistedResponse = router
        .wrap()
        .query_wasm_smart(factory.clone(), &query_msg)
        .unwrap();
    assert!(!res.is_whitelisted, "Contract should not be whitelisted initially");

    // Add contract to whitelist via sudo
    let sudo_msg = SudoMsg::AddContractToWhitelist {
        address: contract_addr.to_string(),
    };
    let res = router.wasm_sudo(factory.clone(), &sudo_msg);
    assert!(res.is_ok(), "Should be able to add contract via sudo");

    // Check that contract is now whitelisted
    let res: IsContractWhitelistedResponse = router
        .wrap()
        .query_wasm_smart(factory.clone(), &query_msg)
        .unwrap();
    assert!(res.is_whitelisted, "Contract should be whitelisted after adding");

    // Remove contract from whitelist via sudo
    let sudo_msg = SudoMsg::RemoveContractFromWhitelist {
        address: contract_addr.to_string(),
    };
    let res = router.wasm_sudo(factory.clone(), &sudo_msg);
    assert!(res.is_ok(), "Should be able to remove contract via sudo");

    // Check that contract is no longer whitelisted
    let res: IsContractWhitelistedResponse = router
        .wrap()
        .query_wasm_smart(factory, &query_msg)
        .unwrap();
    assert!(!res.is_whitelisted, "Contract should not be whitelisted after removal");
}

#[test]
fn test_factory_batch_whitelist_update() {
    use vending_factory::msg::{IsContractWhitelistedResponse, QueryMsg as FactoryQueryMsg, SudoMsg, WhitelistedContractsResponse};

    let vt = vending_minter_template_with_code_ids_template(1);
    let (mut router, _, _) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let factory = vt.collection_response_vec[0].factory.clone().unwrap();

    let contract1 = "contract1234567890abcdefghijklmnopqrstuvwxyz1234567890abcdefgh1";
    let contract2 = "contract1234567890abcdefghijklmnopqrstuvwxyz1234567890abcdefgh2";
    let contract3 = "contract1234567890abcdefghijklmnopqrstuvwxyz1234567890abcdefgh3";

    // Add contract1 first
    let sudo_msg = SudoMsg::AddContractToWhitelist {
        address: contract1.to_string(),
    };
    router.wasm_sudo(factory.clone(), &sudo_msg).unwrap();

    // Batch update: add contract2 and contract3, remove contract1
    let batch_msg = SudoMsg::UpdateContractWhitelist {
        add: vec![contract2.to_string(), contract3.to_string()],
        remove: vec![contract1.to_string()],
    };
    let res = router.wasm_sudo(factory.clone(), &batch_msg);
    assert!(res.is_ok(), "Batch update should succeed");

    // Verify results
    let query_msg1 = FactoryQueryMsg::IsContractWhitelisted {
        address: contract1.to_string(),
    };
    let res1: IsContractWhitelistedResponse = router
        .wrap()
        .query_wasm_smart(factory.clone(), &query_msg1)
        .unwrap();
    assert!(!res1.is_whitelisted, "Contract1 should be removed");

    let query_msg2 = FactoryQueryMsg::IsContractWhitelisted {
        address: contract2.to_string(),
    };
    let res2: IsContractWhitelistedResponse = router
        .wrap()
        .query_wasm_smart(factory.clone(), &query_msg2)
        .unwrap();
    assert!(res2.is_whitelisted, "Contract2 should be added");

    let query_msg3 = FactoryQueryMsg::IsContractWhitelisted {
        address: contract3.to_string(),
    };
    let res3: IsContractWhitelistedResponse = router
        .wrap()
        .query_wasm_smart(factory.clone(), &query_msg3)
        .unwrap();
    assert!(res3.is_whitelisted, "Contract3 should be added");

    // Query all whitelisted contracts
    let query_msg = FactoryQueryMsg::WhitelistedContracts {
        start_after: None,
        limit: None,
    };
    let res: WhitelistedContractsResponse = router
        .wrap()
        .query_wasm_smart(factory, &query_msg)
        .unwrap();

    assert_eq!(res.contracts.len(), 2, "Should have 2 whitelisted contracts");
    assert!(res.contracts.contains(&contract2.to_string()));
    assert!(res.contracts.contains(&contract3.to_string()));
}
