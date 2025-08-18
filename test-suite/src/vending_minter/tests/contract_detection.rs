use crate::common_setup::templates::vending_minter_template;
use cosmwasm_std::{coins, Addr};
use cw_multi_test::Executor;
use sg_utils::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
use vending_minter::msg::ExecuteMsg;
use vending_minter::ContractError;

use crate::common_setup::setup_accounts_and_block::setup_block_time;

const MINT_PRICE: u128 = 100_000_000;

#[test]
fn test_eoa_can_mint() {
    let vt = vending_minter_template(1);
    let (mut router, _creator, buyer) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();

    // Set time after start time to enable minting
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 1, None);

    // EOA address should be able to mint
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        buyer,
        minter_addr,
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(res.is_ok(), "EOA should be able to mint");
}

#[test]
fn test_short_addresses_treated_as_eoa() {
    let vt = vending_minter_template(4); // Need 4 tokens for this test
    let (mut router, _creator, _buyer) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();

    // Create various short address formats (like typical EOAs)
    let short_addresses = vec![
        Addr::unchecked("buyer"),                                 // 5 chars
        Addr::unchecked("simple_string_address"),                 // 21 chars
        Addr::unchecked("cosmos1abc123def456ghi789jkl012mno345"), // 35 chars (typical bech32)
        Addr::unchecked("terra1xyz789abc123def456ghi789jkl012"),  // 34 chars
    ];

    // Set time after start time to enable minting
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 1, None);

    for addr in short_addresses {
        // Fund this address
        router
            .sudo(cw_multi_test::SudoMsg::Bank(
                cw_multi_test::BankSudo::Mint {
                    to_address: addr.to_string(),
                    amount: coins(MINT_PRICE * 2, NATIVE_DENOM),
                },
            ))
            .unwrap();

        // Short addresses should be treated as EOAs and allowed to mint
        let mint_msg = ExecuteMsg::Mint {};
        let res = router.execute_contract(
            addr.clone(),
            minter_addr.clone(),
            &mint_msg,
            &coins(MINT_PRICE, NATIVE_DENOM),
        );

        assert!(
            res.is_ok(),
            "Short address '{}' should be treated as EOA and allowed to mint",
            addr
        );
    }
}

#[test]
fn test_long_addresses_treated_as_contracts() {
    let vt = vending_minter_template(1);
    let (mut router, _creator, _buyer) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();

    // Create a long address that looks like a contract address (>50 chars)
    let long_contract_addr =
        Addr::unchecked("contract1234567890abcdefghijklmnopqrstuvwxyz1234567890abcdefghij");

    // Fund this address
    router
        .sudo(cw_multi_test::SudoMsg::Bank(
            cw_multi_test::BankSudo::Mint {
                to_address: long_contract_addr.to_string(),
                amount: coins(MINT_PRICE * 2, NATIVE_DENOM),
            },
        ))
        .unwrap();

    // Set time after start time to enable minting
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 1, None);

    // Long addresses should be treated as contracts and blocked from minting
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        long_contract_addr.clone(),
        minter_addr,
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );

    // This should fail with ContractsCannotMint error
    assert!(
        res.is_err(),
        "Long address should be treated as contract and blocked from minting"
    );

    let err = res.unwrap_err();
    let contract_err = err.downcast_ref::<ContractError>().unwrap();
    assert_eq!(*contract_err, ContractError::ContractsCannotMint {});
}

#[test]
fn test_admin_mint_to_works_from_any_address() {
    let vt = vending_minter_template(1);
    let (mut router, creator, buyer) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();

    // Set time after start time
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 1, None);

    // Admin mint_to should work regardless of address length
    // Note: This assumes the creator is the admin and can perform mint_to
    let mint_to_msg = ExecuteMsg::MintTo {
        recipient: buyer.to_string(),
    };

    let res = router.execute_contract(
        creator, // Admin performing the action
        minter_addr,
        &mint_to_msg,
        &[],
    );

    assert!(
        res.is_ok(),
        "Admin mint_to should work regardless of caller type"
    );
}

#[test]
fn test_boundary_address_length() {
    let vt = vending_minter_template(2); // Need 2 tokens for this test
    let (mut router, _creator, _buyer) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();

    // Test addresses right at the boundary (50 chars)
    let boundary_addr_49 = Addr::unchecked("1234567890123456789012345678901234567890123456789"); // 49 chars - should be EOA
    let boundary_addr_51 = Addr::unchecked("123456789012345678901234567890123456789012345678901"); // 51 chars - should be contract

    // Set time after start time to enable minting
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 1, None);

    // Fund both addresses
    for addr in [&boundary_addr_49, &boundary_addr_51] {
        router
            .sudo(cw_multi_test::SudoMsg::Bank(
                cw_multi_test::BankSudo::Mint {
                    to_address: addr.to_string(),
                    amount: coins(MINT_PRICE * 2, NATIVE_DENOM),
                },
            ))
            .unwrap();
    }

    // 49-char address should work (treated as EOA)
    let mint_msg = ExecuteMsg::Mint {};
    let res_49 = router.execute_contract(
        boundary_addr_49,
        minter_addr.clone(),
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(res_49.is_ok(), "49-char address should be treated as EOA");

    // 51-char address should fail (treated as contract)
    let mint_msg = ExecuteMsg::Mint {};
    let res_51 = router.execute_contract(
        boundary_addr_51,
        minter_addr,
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );
    assert!(
        res_51.is_err(),
        "51-char address should be treated as contract"
    );

    let err = res_51.unwrap_err();
    let contract_err = err.downcast_ref::<ContractError>().unwrap();
    assert_eq!(*contract_err, ContractError::ContractsCannotMint {});
}

#[test]
fn test_real_contract_detected_by_contract_info_query() {
    let vt = vending_minter_template(1);
    let (mut router, _creator, _buyer) = (vt.router, vt.accts.creator, vt.accts.buyer);
    let minter_addr = vt.collection_response_vec[0].minter.clone().unwrap();

    // Use the actual minter contract address (which is a real contract but short address)
    // This tests the ContractInfo query fallback since the minter address is likely <50 chars
    // but is definitely a contract that should be blocked from minting

    // Set time after start time to enable minting
    setup_block_time(&mut router, GENESIS_MINT_START_TIME + 1, None);

    // Fund the minter contract address so it can attempt to mint
    router
        .sudo(cw_multi_test::SudoMsg::Bank(
            cw_multi_test::BankSudo::Mint {
                to_address: minter_addr.to_string(),
                amount: coins(MINT_PRICE * 2, NATIVE_DENOM),
            },
        ))
        .unwrap();

    // The minter contract trying to mint from itself should be blocked
    let mint_msg = ExecuteMsg::Mint {};
    let res = router.execute_contract(
        minter_addr.clone(),
        minter_addr,
        &mint_msg,
        &coins(MINT_PRICE, NATIVE_DENOM),
    );

    // This should fail with ContractsCannotMint error because the ContractInfo
    // query should detect that minter_addr is a contract, even if it's <50 chars
    assert!(
        res.is_err(),
        "Real contract address should be blocked from minting via ContractInfo query"
    );

    let err = res.unwrap_err();
    let contract_err = err.downcast_ref::<ContractError>().unwrap();
    assert_eq!(*contract_err, ContractError::ContractsCannotMint {});
}
