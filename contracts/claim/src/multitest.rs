#![cfg(test)]
use crate::error::ContractError;
use crate::msg::MarketplaceResponse;
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
    .with_sudo(sg_marketplace::sudo::sudo)
    .with_reply(sg_marketplace::execute::reply);

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
    use super::*;
    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
    use cosmwasm_std::{coin, coins, Addr, Coin, Decimal, Empty, Uint128};
    use cw721::{Cw721QueryMsg, OwnerOfResponse};
    use cw_controllers::AdminResponse;
    use cw_multi_test::Executor;
    use cw_utils::Duration;
    use sg721::msg::{InstantiateMsg as Sg721InstantiateMsg, RoyaltyInfoResponse};
    use sg721::state::CollectionInfo;
    use sg_controllers::HooksResponse;
    use sg_marketplace::msg::{ExecuteMsg as MktExecuteMsg, QueryMsg as MktQueryMsg, SudoMsg};
    use sg_marketplace::state::SaleType;
    use sg_marketplace::ExpiryRange;
    use sg_multi_test::StargazeApp;
    use sg_std::NATIVE_DENOM;

    const TOKEN_ID: u32 = 123;
    const CREATION_FEE: u128 = 1_000_000_000;
    const INITIAL_BALANCE: u128 = 2000;
    // Governance parameters
    const TRADING_FEE_BPS: u64 = 200; // 2%
    const MIN_EXPIRY: u64 = 24 * 60 * 60; // 24 hours (in seconds)
    const MAX_EXPIRY: u64 = 180 * 24 * 60 * 60; // 6 months (in seconds)
    const MAX_FINDERS_FEE_BPS: u64 = 1000; // 10%
    const BID_REMOVAL_REWARD_BPS: u64 = 500; // 5%

    // Instantiates all needed contracts for testing
    fn setup_contracts(
        router: &mut StargazeApp,
        creator: &Addr,
    ) -> Result<(Addr, Addr, Addr), ContractError> {
        // Instantiate marketplace contract
        let marketplace_id = router.store_code(contract_marketplace());
        let msg = sg_marketplace::msg::InstantiateMsg {
            operators: vec!["operator".to_string()],
            trading_fee_bps: TRADING_FEE_BPS,
            ask_expiry: ExpiryRange::new(MIN_EXPIRY, MAX_EXPIRY),
            bid_expiry: ExpiryRange::new(MIN_EXPIRY, MAX_EXPIRY),
            sale_hook: None,
            max_finders_fee_bps: MAX_FINDERS_FEE_BPS,
            min_price: Uint128::from(5u128),
            stale_bid_duration: Duration::Time(100),
            bid_removal_reward_bps: BID_REMOVAL_REWARD_BPS,
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
            marketplace_addr: Some(marketplace_addr.to_string()),
            admin: Some(creator.to_string()),
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
        let add_hook_msg = SudoMsg::AddSaleHook {
            hook: claims_addr.to_string(),
        };
        let res = router.wasm_sudo(marketplace_addr.clone(), &add_hook_msg);
        assert!(res.is_ok());

        // query to check if hook was added
        let query_hooks_msg = MktQueryMsg::SaleHooks {};
        let res: HooksResponse = router
            .wrap()
            .query_wasm_smart(marketplace_addr.clone(), &query_hooks_msg)
            .unwrap();
        assert_eq!(res.hooks, vec![claims_addr.to_string()]);

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
        let set_ask = MktExecuteMsg::SetAsk {
            sale_type: SaleType::FixedPrice,
            collection: collection_addr.to_string(),
            token_id: TOKEN_ID,
            price: coin(100, NATIVE_DENOM),
            funds_recipient: None,
            reserve_for: None,
            expires: router.block_info().time.plus_seconds(MIN_EXPIRY + 1),
            finders_fee_bps: Some(0),
        };
        let res = router.execute_contract(creator.clone(), marketplace_addr.clone(), &set_ask, &[]);
        assert!(res.is_ok());

        // Bidder makes bid
        let set_bid_msg = MktExecuteMsg::SetBid {
            collection: collection_addr.to_string(),
            token_id: TOKEN_ID,
            finders_fee_bps: None,
            expires: router.block_info().time.plus_seconds(MIN_EXPIRY + 1),
            finder: None,
        };
        let res = router.execute_contract(
            bidder.clone(),
            marketplace_addr,
            &set_bid_msg,
            &coins(100, NATIVE_DENOM),
        );
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res.events.len(), 11);
        assert_eq!("wasm-finalize-sale", res.events[3].ty);
        assert_eq!("claim_buy_nft", res.events[10].attributes[1].value);

        // Check NFT is transferred
        let query_owner_msg = Cw721QueryMsg::OwnerOf {
            token_id: TOKEN_ID.to_string(),
            include_expired: None,
        };
        let res: OwnerOfResponse = router
            .wrap()
            .query_wasm_smart(collection_addr, &query_owner_msg)
            .unwrap();
        assert_eq!(res.owner, bidder.to_string());
    }

    #[test]
    fn check_update_admin_and_queries() {
        let mut router = custom_mock_app();
        // Setup intial accounts
        let (_owner, _, creator) = setup_accounts(&mut router).unwrap();
        // Instantiate and configure contracts
        let (_, _, claims_addr) = setup_contracts(&mut router, &creator).unwrap();

        let msg = ExecuteMsg::UpdateAdmin {
            admin: Some("new_admin".to_string()),
        };
        let res = router.execute_contract(creator.clone(), claims_addr.clone(), &msg, &[]);
        assert!(res.is_ok());

        let query_msg = QueryMsg::Admin {};
        let res: AdminResponse = router
            .wrap()
            .query_wasm_smart(claims_addr.clone(), &query_msg)
            .unwrap();
        assert_eq!(res.admin, Some("new_admin".to_string()));

        let query_msg = QueryMsg::Marketplace {};
        let res: MarketplaceResponse = router
            .wrap()
            .query_wasm_smart(claims_addr, &query_msg)
            .unwrap();
        assert_eq!(res.marketplace, Some(Addr::unchecked("contract0")));
    }
}
