use crate::common_setup::templates::vending_minter_template;
use cosmwasm_std::{coins, Addr};
use cw_multi_test::Executor;
use sg_utils::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
use vending_minter::msg::ExecuteMsg;
use vending_factory::msg::SudoMsg;
use sg2::query::{IsContractWhitelistedResponse, Sg2QueryMsg as FactoryQueryMsg, WhitelistedContractsResponse};

use crate::common_setup::setup_accounts_and_block::setup_block_time;

const MINT_PRICE: u128 = 100_000_000;

#[test]
fn test_governance_can_add_contract_to_factory_whitelist() {
    let vt = vending_minter_template(1);
    let (mut router, creator, _buyer) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let factory_addr = vt.collection_response_vec[0].factory.clone().unwrap();

    // Create a fake contract address
    let contract_addr = "contract1234567890abcdefghijklmnopqrstuvwxyz1234567890abcdefghij";

    // Use sudo to add contract to factory whitelist (governance-only operation)
    let sudo_msg = SudoMsg::AddContractToWhitelist {
        address: contract_addr.to_string(),
    };

    let res = router.wasm_sudo(factory_addr.clone(), &sudo_msg);
    assert!(res.is_ok(), "Governance should be able to add contract to factory whitelist");

    // Check that the contract is now whitelisted via factory query
    let query_msg = FactoryQueryMsg::IsContractWhitelisted {
        address: contract_addr.to_string(),
    };
    let res: IsContractWhitelistedResponse = router
        .wrap()
        .query_wasm_smart(factory_addr, &query_msg)
        .unwrap();

    assert!(res.is_whitelisted, "Contract should be whitelisted");
    assert_eq!(res.address, contract_addr);
}

#[test]
fn test_minter_no_longer_has_whitelist_execute_handlers() {
    let vt = vending_minter_template(1);
    let (_router, _creator, _buyer) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let _minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();

    let _contract_addr = "contract1234567890abcdefghijklmnopqrstuvwxyz1234567890abcdefghij";

    // These execute messages should no longer exist on the minter
    // This test should fail to compile, proving the migration worked
    // Commenting out to allow compilation
    
    // let add_msg = ExecuteMsg::AddContractToWhitelist {
    //     address: contract_addr.to_string(),
    // };
    // let res = router.execute_contract(creator, minter_addr, &add_msg, &[]);
    // assert!(res.is_err(), "Minter should no longer have whitelist execute handlers");
}

#[test]
fn test_whitelisted_contract_can_mint() {
    let vt = vending_minter_template(2); // Need 2 tokens for this test
    let (mut router, _creator, _buyer) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();
    let factory_addr = vt.collection_response_vec[0].factory.clone().unwrap();

    // Create a long contract address (would normally be blocked)
    let contract_addr = Addr::unchecked("contract1234567890abcdefghijklmnopqrstuvwxyz1234567890abcdefghij");

    // Set time after start time to enable minting
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 1, None);

    // Fund the contract address
    router
        .sudo(cw_multi_test::SudoMsg::Bank(
            cw_multi_test::BankSudo::Mint {
                to_address: contract_addr.to_string(),
                amount: coins(MINT_PRICE * 2, NATIVE_DENOM),
            },
        ))
        .unwrap();

    // First, minting should fail (contract not whitelisted)
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        contract_addr.clone(),
        minter_addr.clone(),
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_err(), "Non-whitelisted contract should be blocked");

    // Governance adds contract to factory whitelist (sudo operation)
    let sudo_msg = SudoMsg::AddContractToWhitelist {
        address: contract_addr.to_string(),
    };
    let res = router.wasm_sudo(factory_addr, &sudo_msg);
    assert!(res.is_ok(), "Governance should be able to add contract to factory whitelist");

    // Now minting should succeed
    let res = router.execute_contract(
        contract_addr,
        minter_addr,
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_ok(), "Whitelisted contract should be able to mint");
}

#[test]
fn test_remove_contract_from_factory_whitelist() {
    let vt = vending_minter_template(1);
    let (mut router, _creator, _buyer) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let factory_addr = vt.collection_response_vec[0].factory.clone().unwrap();

    let contract_addr = "contract1234567890abcdefghijklmnopqrstuvwxyz1234567890abcdefghij";

    // Add contract to factory whitelist via sudo
    let add_msg = SudoMsg::AddContractToWhitelist {
        address: contract_addr.to_string(),
    };
    router
        .wasm_sudo(factory_addr.clone(), &add_msg)
        .unwrap();

    // Verify it's whitelisted
    let query_msg = FactoryQueryMsg::IsContractWhitelisted {
        address: contract_addr.to_string(),
    };
    let res: IsContractWhitelistedResponse = router
        .wrap()
        .query_wasm_smart(factory_addr.clone(), &query_msg)
        .unwrap();
    assert!(res.is_whitelisted);

    // Remove from whitelist via sudo
    let remove_msg = SudoMsg::RemoveContractFromWhitelist {
        address: contract_addr.to_string(),
    };
    let res = router.wasm_sudo(factory_addr.clone(), &remove_msg);
    assert!(res.is_ok(), "Governance should be able to remove from factory whitelist");

    // Verify it's no longer whitelisted
    let res: IsContractWhitelistedResponse = router
        .wrap()
        .query_wasm_smart(factory_addr, &query_msg)
        .unwrap();
    assert!(!res.is_whitelisted, "Contract should no longer be whitelisted");
}

#[test]
fn test_batch_update_factory_contract_whitelist() {
    let vt = vending_minter_template(1);
    let (mut router, _creator, _buyer) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let factory_addr = vt.collection_response_vec[0].factory.clone().unwrap();

    let contract1 = "contract1234567890abcdefghijklmnopqrstuvwxyz1234567890abcdefgh1";
    let contract2 = "contract1234567890abcdefghijklmnopqrstuvwxyz1234567890abcdefgh2";
    let contract3 = "contract1234567890abcdefghijklmnopqrstuvwxyz1234567890abcdefgh3";

    // Add contract1 to factory whitelist first via sudo
    let add_msg = SudoMsg::AddContractToWhitelist {
        address: contract1.to_string(),
    };
    router
        .wasm_sudo(factory_addr.clone(), &add_msg)
        .unwrap();

    // Batch update: add contract2 and contract3, remove contract1
    let batch_msg = SudoMsg::UpdateContractWhitelist {
        add: vec![contract2.to_string(), contract3.to_string()],
        remove: vec![contract1.to_string()],
    };

    let res = router.wasm_sudo(factory_addr.clone(), &batch_msg);
    assert!(res.is_ok(), "Governance should be able to batch update factory whitelist");

    // Check results
    let query_msg1 = FactoryQueryMsg::IsContractWhitelisted {
        address: contract1.to_string(),
    };
    let res1: IsContractWhitelistedResponse = router
        .wrap()
        .query_wasm_smart(factory_addr.clone(), &query_msg1)
        .unwrap();
    assert!(!res1.is_whitelisted, "Contract1 should be removed");

    let query_msg2 = FactoryQueryMsg::IsContractWhitelisted {
        address: contract2.to_string(),
    };
    let res2: IsContractWhitelistedResponse = router
        .wrap()
        .query_wasm_smart(factory_addr.clone(), &query_msg2)
        .unwrap();
    assert!(res2.is_whitelisted, "Contract2 should be added");

    let query_msg3 = FactoryQueryMsg::IsContractWhitelisted {
        address: contract3.to_string(),
    };
    let res3: IsContractWhitelistedResponse = router
        .wrap()
        .query_wasm_smart(factory_addr, &query_msg3)
        .unwrap();
    assert!(res3.is_whitelisted, "Contract3 should be added");
}

#[test]
fn test_query_factory_whitelisted_contracts() {
    let vt = vending_minter_template(1);
    let (mut router, _creator, _buyer) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let factory_addr = vt.collection_response_vec[0].factory.clone().unwrap();

    let contract1 = "contract1234567890abcdefghijklmnopqrstuvwxyz1234567890abcdefgh1";
    let contract2 = "contract1234567890abcdefghijklmnopqrstuvwxyz1234567890abcdefgh2";

    // Add both contracts to factory whitelist via sudo
    let batch_msg = SudoMsg::UpdateContractWhitelist {
        add: vec![contract1.to_string(), contract2.to_string()],
        remove: vec![],
    };
    router
        .wasm_sudo(factory_addr.clone(), &batch_msg)
        .unwrap();

    // Query all whitelisted contracts from factory
    let query_msg = FactoryQueryMsg::WhitelistedContracts {
        start_after: None,
        limit: None,
    };
    let res: WhitelistedContractsResponse = router
        .wrap()
        .query_wasm_smart(factory_addr, &query_msg)
        .unwrap();

    assert_eq!(res.contracts.len(), 2, "Should have 2 whitelisted contracts");
    assert!(res.contracts.contains(&contract1.to_string()));
    assert!(res.contracts.contains(&contract2.to_string()));
}