#![cfg(test)]
use crate::error::ContractError;
use cw721_base::msg::{ExecuteMsg as Cw721ExecuteMsg, MintMsg};
use cw_multi_test::{BankSudo, Contract, ContractWrapper, SudoMsg as CwSudoMsg};
use sg_multi_test::StargazeApp;
use sg_std::StargazeMsgWrapper;

fn custom_mock_app() -> StargazeApp {
    StargazeApp::default()
}

pub fn contract_marketplace() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        sg_marketplace::execute::execute,
        sg_marketplace::execute::instantiate,
        sg_marketplace::query::query,
    )
    .with_sudo(sg_marketplace::sudo::sudo);
    Box::new(contract)
}

pub fn contract_sg721() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        sg721::contract::execute,
        sg721::contract::instantiate,
        sg721::contract::query,
    );
    Box::new(contract)
}

pub fn contract_claim() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

#[cfg(test)]
mod tests {
    use cw_multi_test::Executor;
    use sg_marketplace::msg::{ExecuteMsg, SudoMsg};

    use crate::msg::InstantiateMsg;

    use super::*;
    use cosmwasm_std::{coin, coins, Addr, Coin, Decimal, Empty};
    use sg721::msg::{InstantiateMsg as Sg721InstantiateMsg, RoyaltyInfoResponse};
    use sg721::state::CollectionInfo;
    use sg_multi_test::StargazeApp;
    use sg_std::NATIVE_DENOM;

    const TOKEN_ID: u32 = 123;
    const CREATION_FEE: u128 = 1_000_000_000;
    const INITIAL_BALANCE: u128 = 2000;
    // Governance parameters
    const TRADING_FEE_PERCENT: u32 = 2; // 2%
    const MIN_EXPIRY: u64 = 24 * 60 * 60; // 24 hours (in seconds)
    const MAX_EXPIRY: u64 = 180 * 24 * 60 * 60; // 6 months (in seconds)

    // Instantiates all needed contracts for testing
    fn setup_contracts(
        router: &mut StargazeApp,
        creator: &Addr,
    ) -> Result<(Addr, Addr, Addr), ContractError> {
        // Instantiate marketplace contract
        let marketplace_id = router.store_code(contract_marketplace());
        let msg = sg_marketplace::msg::InstantiateMsg {
            operators: vec!["operator".to_string()],
            trading_fee_percent: TRADING_FEE_PERCENT,
            ask_expiry: (MIN_EXPIRY, MAX_EXPIRY),
            bid_expiry: (MIN_EXPIRY, MAX_EXPIRY),
        };
        let marketplace_addr = router
            .instantiate_contract(
                marketplace_id,
                creator.clone(),
                &msg,
                &[],
                "Marketplace",
                None,
            )
            .unwrap();

        // Setup media contract
        let sg721_id = router.store_code(contract_sg721());
        let msg = Sg721InstantiateMsg {
            name: String::from("Test Coin"),
            symbol: String::from("TEST"),
            minter: creator.to_string(),
            collection_info: CollectionInfo {
                creator: creator.to_string(),
                description: String::from("Stargaze Monkeys"),
                image: "ipfs://bafybeigi3bwpvyvsmnbj46ra4hyffcxdeaj6ntfk5jpic5mx27x6ih2qvq/images/1.png".to_string(),
                external_link: Some("https://example.com/external.html".to_string()),
                royalty_info: Some(RoyaltyInfoResponse {
                    payment_address: creator.to_string(),
                    share: Decimal::percent(10),
                }),
            },
        };
        let collection = router
            .instantiate_contract(
                sg721_id,
                creator.clone(),
                &msg,
                &coins(CREATION_FEE, NATIVE_DENOM),
                "NFT",
                None,
            )
            .unwrap();

        // setup claim contract
        let claim_id = router.store_code(contract_claim());
        let msg = InstantiateMsg {
            marketplace_addr: marketplace_addr.to_string(),
        };
        let claims = router
            .instantiate_contract(claim_id, creator.clone(), &msg, &[], "claims", None)
            .unwrap();

        Ok((marketplace_addr, collection, claims))
    }

    // Intializes accounts with balances
    fn setup_accounts(router: &mut StargazeApp) -> Result<(Addr, Addr, Addr), ContractError> {
        let owner: Addr = Addr::unchecked("owner");
        let bidder: Addr = Addr::unchecked("bidder");
        let creator: Addr = Addr::unchecked("creator");
        let creator_funds: Vec<Coin> = coins(CREATION_FEE, NATIVE_DENOM);
        let funds: Vec<Coin> = coins(INITIAL_BALANCE, NATIVE_DENOM);
        router
            .sudo(CwSudoMsg::Bank({
                BankSudo::Mint {
                    to_address: owner.to_string(),
                    amount: funds.clone(),
                }
            }))
            .map_err(|err| println!("{:?}", err))
            .ok();
        router
            .sudo(CwSudoMsg::Bank({
                BankSudo::Mint {
                    to_address: bidder.to_string(),
                    amount: funds.clone(),
                }
            }))
            .map_err(|err| println!("{:?}", err))
            .ok();
        router
            .sudo(CwSudoMsg::Bank({
                BankSudo::Mint {
                    to_address: creator.to_string(),
                    amount: creator_funds.clone(),
                }
            }))
            .map_err(|err| println!("{:?}", err))
            .ok();

        // Check native balances
        let owner_native_balances = router.wrap().query_all_balances(owner.clone()).unwrap();
        assert_eq!(owner_native_balances, funds);
        let bidder_native_balances = router.wrap().query_all_balances(bidder.clone()).unwrap();
        assert_eq!(bidder_native_balances, funds);
        let creator_native_balances = router.wrap().query_all_balances(creator.clone()).unwrap();
        assert_eq!(creator_native_balances, creator_funds);

        Ok((owner, bidder, creator))
    }

    // Mints an NFT for a creator
    fn mint_nft_for_creator(router: &mut StargazeApp, creator: &Addr, nft_contract_addr: &Addr) {
        let mint_for_creator_msg = Cw721ExecuteMsg::Mint(MintMsg {
            token_id: TOKEN_ID.to_string(),
            owner: creator.clone().to_string(),
            token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
            extension: Empty {},
        });
        let res = router.execute_contract(
            creator.clone(),
            nft_contract_addr.clone(),
            &mint_for_creator_msg,
            &[],
        );
        assert!(res.is_ok());
    }

    #[test]
    fn check_hook_fired() {
        let mut router = custom_mock_app();
        // Setup intial accounts
        let (_owner, bidder, creator) = setup_accounts(&mut router).unwrap();
        // Instantiate and configure contracts
        let (marketplace_addr, collection_addr, claims_addr) =
            setup_contracts(&mut router, &creator).unwrap();

        // setup sale finalized hook
        let add_hook_msg = SudoMsg::AddSaleFinalizedHook {
            hook: claims_addr.to_string(),
        };
        let res = router.wasm_sudo(marketplace_addr.clone(), &add_hook_msg);
        assert!(res.is_ok());

        // Mint NFT for creator
        mint_nft_for_creator(&mut router, &creator, &collection_addr);

        // Creator Authorizes NFT
        let approve_msg = Cw721ExecuteMsg::<Empty>::Approve {
            spender: marketplace_addr.to_string(),
            token_id: TOKEN_ID.to_string(),
            expires: None,
        };
        let res =
            router.execute_contract(creator.clone(), collection_addr.clone(), &approve_msg, &[]);
        assert!(res.is_ok());

        // An asking price is made by the creator
        let set_ask = ExecuteMsg::SetAsk {
            collection: collection_addr.to_string(),
            token_id: TOKEN_ID,
            price: coin(100, NATIVE_DENOM),
            funds_recipient: None,
            expires: router.block_info().time.plus_seconds(MIN_EXPIRY + 1),
        };
        let res = router.execute_contract(creator.clone(), marketplace_addr.clone(), &set_ask, &[]);
        assert!(res.is_ok());

        // Bidder makes bid
        let set_bid_msg = ExecuteMsg::SetBid {
            collection: collection_addr.to_string(),
            token_id: TOKEN_ID,
            expires: router.block_info().time.plus_seconds(MIN_EXPIRY + 1),
        };
        let res = router.execute_contract(
            bidder,
            marketplace_addr,
            &set_bid_msg,
            &coins(100, NATIVE_DENOM),
        );
        // TODO: this fails, maybe multitest doesn't support hooks yet?
        println!("{:?}", res);
        assert!(res.is_ok());

        // // Check NFT is transferred
        // let query_owner_msg = Cw721QueryMsg::OwnerOf {
        //     token_id: TOKEN_ID.to_string(),
        //     include_expired: None,
        // };
        // let res: OwnerOfResponse = router
        //     .wrap()
        //     .query_wasm_smart(collection_addr, &query_owner_msg)
        //     .unwrap();
        // assert_eq!(res.owner, bidder.to_string());
    }
}
