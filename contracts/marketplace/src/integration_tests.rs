#![cfg(test)]
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, QueryMsg};
use crate::state::{Ask, Bid};
use cosmwasm_std::{Addr, Empty};
use cw721::{Cw721QueryMsg, OwnerOfResponse};
use cw721_base::msg::{ExecuteMsg as Cw721ExecuteMsg, MintMsg};
use cw_multi_test::{App, BankSudo, Contract, ContractWrapper, Executor, SudoMsg};

fn mock_app() -> App {
    App::default()
}

pub fn contract_nft_marketplace() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

pub fn contract_sg721() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        sg721::contract::execute,
        sg721::contract::instantiate,
        sg721::contract::query,
    );
    Box::new(contract)
}

pub fn contract_cw721() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw721_metadata_onchain::entry::execute,
        cw721_metadata_onchain::entry::instantiate,
        cw721_metadata_onchain::entry::query,
    );
    Box::new(contract)
}

#[cfg(test)]
mod tests {
    use crate::msg::BidResponse;

    use super::*;
    use cosmwasm_std::{coin, coins, Coin, Decimal};
    use sg721::state::{Config, RoyaltyInfo};

    const TOKEN_ID: &str = "123";
    const NATIVE_TOKEN_DENOM: &str = "ustars";
    const INITIAL_BALANCE: u128 = 2000;

    // Instantiates all needed contracts for testing
    fn setup_contracts(router: &mut App, creator: &Addr) -> Result<(Addr, Addr), ContractError> {
        // Instantiate marketplace contract
        let marketplace_id = router.store_code(contract_nft_marketplace());
        let msg = crate::msg::InstantiateMsg {};
        let nft_marketplace_addr = router
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
        let msg = sg721::msg::InstantiateMsg {
            name: String::from("Test Coin"),
            symbol: String::from("TEST"),
            minter: creator.to_string(),
            config: Some(Config {
                contract_uri: Some(String::from("https://bafyreibvxty5gjyeedk7or7tahyrzgbrwjkolpairjap3bmegvcjdipt74.ipfs.dweb.link/metadata.json")),
                creator: creator.clone(),
                royalties: Some(RoyaltyInfo {
                    payment_address: creator.clone(),
                    share: Decimal::percent(10),
                }),
            }),
        };
        let nft_contract_addr = router
            .instantiate_contract(sg721_id, creator.clone(), &msg, &[], "NFT", None)
            .unwrap();

        Ok((nft_marketplace_addr, nft_contract_addr))
    }

    // Intializes accounts with balances
    fn setup_accounts(router: &mut App) -> Result<(Addr, Addr, Addr), ContractError> {
        let owner: Addr = Addr::unchecked("owner");
        let bidder: Addr = Addr::unchecked("bidder");
        let creator: Addr = Addr::unchecked("creator");
        let funds: Vec<Coin> = coins(INITIAL_BALANCE, NATIVE_TOKEN_DENOM);
        router
            .sudo(SudoMsg::Bank({
                BankSudo::Mint {
                    to_address: owner.to_string(),
                    amount: funds.clone(),
                }
            }))
            .map_err(|err| println!("{:?}", err))
            .ok();
        router
            .sudo(SudoMsg::Bank({
                BankSudo::Mint {
                    to_address: bidder.to_string(),
                    amount: funds.clone(),
                }
            }))
            .map_err(|err| println!("{:?}", err))
            .ok();
        router
            .sudo(SudoMsg::Bank({
                BankSudo::Mint {
                    to_address: creator.to_string(),
                    amount: funds.clone(),
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
        assert_eq!(creator_native_balances, funds);

        Ok((owner, bidder, creator))
    }

    // Mints an NFT for a creator
    fn mint_nft_for_creator(router: &mut App, creator: &Addr, nft_contract_addr: &Addr) {
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
    fn happy_path() {
        let mut router = mock_app();

        // Setup intial accounts
        let (_owner, bidder, creator) = setup_accounts(&mut router).unwrap();

        // Instantiate and configure contracts
        let (nft_marketplace_addr, nft_contract_addr) =
            setup_contracts(&mut router, &creator).unwrap();

        // Mint NFT for creator
        mint_nft_for_creator(&mut router, &creator, &nft_contract_addr);

        // Creator Authorizes NFT
        let approve_msg = Cw721ExecuteMsg::<Empty>::Approve {
            spender: nft_marketplace_addr.to_string(),
            token_id: TOKEN_ID.to_string(),
            expires: None,
        };
        let res = router.execute_contract(
            creator.clone(),
            nft_contract_addr.clone(),
            &approve_msg,
            &[],
        );
        assert!(res.is_ok());

        // An asking price is made by the creator
        let ask = Ask {
            amount: coin(110, NATIVE_TOKEN_DENOM),
        };
        let set_ask = ExecuteMsg::SetAsk {
            collection: nft_contract_addr.clone(),
            token_id: TOKEN_ID.to_string(),
            ask,
        };
        let res =
            router.execute_contract(creator.clone(), nft_marketplace_addr.clone(), &set_ask, &[]);
        assert!(res.is_ok());

        // Bidder makes bid
        let bid = Bid {
            amount: coin(100, NATIVE_TOKEN_DENOM),
            bidder: bidder.clone(),
            recipient: creator.clone(),
        };
        let set_bid_msg = ExecuteMsg::SetBid {
            collection: nft_contract_addr.clone(),
            token_id: TOKEN_ID.to_string(),
            bid: bid.clone(),
        };
        let res = router.execute_contract(
            bidder.clone(),
            nft_marketplace_addr.clone(),
            &set_bid_msg,
            &coins(100, NATIVE_TOKEN_DENOM),
        );
        assert!(res.is_ok());

        // Check contract has been paid
        let contract_balances = router
            .wrap()
            .query_all_balances(nft_marketplace_addr.clone())
            .unwrap();
        assert_eq!(contract_balances, coins(100, NATIVE_TOKEN_DENOM));

        // Check creator hasn't been paid yet
        let creator_native_balances = router.wrap().query_all_balances(creator.clone()).unwrap();
        assert_eq!(
            creator_native_balances,
            coins(INITIAL_BALANCE, NATIVE_TOKEN_DENOM)
        );

        // Creator accepts bid
        let accept_bid_msg = ExecuteMsg::AcceptBid {
            collection: nft_contract_addr.clone(),
            token_id: TOKEN_ID.to_string(),
            bid,
        };
        let res =
            router.execute_contract(creator.clone(), nft_marketplace_addr, &accept_bid_msg, &[]);
        assert!(res.is_ok());

        // Check money is transfered
        let creator_native_balances = router.wrap().query_all_balances(creator).unwrap();
        assert_eq!(
            creator_native_balances,
            coins(INITIAL_BALANCE + 100, NATIVE_TOKEN_DENOM)
        );
        let bidder_native_balances = router.wrap().query_all_balances(bidder.clone()).unwrap();
        assert_eq!(
            bidder_native_balances,
            coins(INITIAL_BALANCE - 100, NATIVE_TOKEN_DENOM)
        );

        // Check NFT is transferred
        let query_owner_msg = Cw721QueryMsg::OwnerOf {
            token_id: TOKEN_ID.to_string(),
            include_expired: None,
        };
        let res: OwnerOfResponse = router
            .wrap()
            .query_wasm_smart(nft_contract_addr, &query_owner_msg)
            .unwrap();
        assert_eq!(res.owner, bidder.to_string());
    }

    #[test]
    fn alternative_cw721_happy_path() {
        let mut router = mock_app();

        // Setup intial accounts, marketplace contract
        let (_, bidder, creator) = setup_accounts(&mut router).unwrap();
        let (nft_marketplace_addr, _) = setup_contracts(&mut router, &creator).unwrap();

        // Upload and instantiate cw721_metadata_onchain contract
        let cw721_id = router.store_code(contract_cw721());
        let msg = cw721_metadata_onchain::InstantiateMsg {
            name: String::from("Test Coin"),
            symbol: String::from("TEST"),
            minter: creator.to_string(),
        };
        let nft_contract_addr = router
            .instantiate_contract(cw721_id, creator.clone(), &msg, &[], "NFT", None)
            .unwrap();

        // Mint NFT
        let mint_for_creator_msg =
            cw721_metadata_onchain::ExecuteMsg::Mint(cw721_metadata_onchain::MintMsg {
                token_id: TOKEN_ID.to_string(),
                owner: creator.to_string(),
                token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
                extension: None,
            });
        let res = router.execute_contract(
            creator.clone(),
            nft_contract_addr.clone(),
            &mint_for_creator_msg,
            &[],
        );
        assert!(res.is_ok());

        // Creator Authorizes NFT
        let approve_msg = Cw721ExecuteMsg::<Empty>::Approve {
            spender: nft_marketplace_addr.to_string(),
            token_id: TOKEN_ID.to_string(),
            expires: None,
        };
        let res = router.execute_contract(
            creator.clone(),
            nft_contract_addr.clone(),
            &approve_msg,
            &[],
        );
        assert!(res.is_ok());

        // An asking price is made by the creator
        let ask = Ask {
            amount: coin(110, NATIVE_TOKEN_DENOM),
        };
        let set_ask = ExecuteMsg::SetAsk {
            collection: nft_contract_addr.clone(),
            token_id: TOKEN_ID.to_string(),
            ask,
        };
        let res =
            router.execute_contract(creator.clone(), nft_marketplace_addr.clone(), &set_ask, &[]);
        assert!(res.is_ok());

        // Bidder makes bid
        let bid = Bid {
            amount: coin(100, NATIVE_TOKEN_DENOM),
            bidder: bidder.clone(),
            recipient: creator.clone(),
        };
        let set_bid_msg = ExecuteMsg::SetBid {
            collection: nft_contract_addr.clone(),
            token_id: TOKEN_ID.to_string(),
            bid: bid.clone(),
        };
        let res = router.execute_contract(
            bidder.clone(),
            nft_marketplace_addr.clone(),
            &set_bid_msg,
            &coins(100, NATIVE_TOKEN_DENOM),
        );
        assert!(res.is_ok());

        // Check contract has been paid
        let contract_balances = router
            .wrap()
            .query_all_balances(nft_marketplace_addr.clone())
            .unwrap();
        assert_eq!(contract_balances, coins(100, NATIVE_TOKEN_DENOM));

        // Check creator hasn't been paid yet
        let creator_native_balances = router.wrap().query_all_balances(creator.clone()).unwrap();
        assert_eq!(
            creator_native_balances,
            coins(INITIAL_BALANCE, NATIVE_TOKEN_DENOM)
        );

        // Creator accepts bid
        let accept_bid_msg = ExecuteMsg::AcceptBid {
            collection: nft_contract_addr.clone(),
            token_id: TOKEN_ID.to_string(),
            bid,
        };
        let res =
            router.execute_contract(creator.clone(), nft_marketplace_addr, &accept_bid_msg, &[]);
        assert!(res.is_ok());

        // Check money is transfered
        let creator_native_balances = router.wrap().query_all_balances(creator).unwrap();
        assert_eq!(
            creator_native_balances,
            coins(INITIAL_BALANCE + 100, NATIVE_TOKEN_DENOM)
        );
        let bidder_native_balances = router.wrap().query_all_balances(bidder.clone()).unwrap();
        assert_eq!(
            bidder_native_balances,
            coins(INITIAL_BALANCE - 100, NATIVE_TOKEN_DENOM)
        );

        // Check NFT is transferred
        let query_owner_msg = Cw721QueryMsg::OwnerOf {
            token_id: TOKEN_ID.to_string(),
            include_expired: None,
        };
        let res: OwnerOfResponse = router
            .wrap()
            .query_wasm_smart(nft_contract_addr, &query_owner_msg)
            .unwrap();
        assert_eq!(res.owner, bidder.to_string());
    }

    #[test]
    fn auto_accept_bid() {
        let mut router = mock_app();

        // Setup intial accounts
        let (_owner, bidder, creator) = setup_accounts(&mut router).unwrap();

        // Instantiate and configure contracts
        let (nft_marketplace_addr, nft_contract_addr) =
            setup_contracts(&mut router, &creator).unwrap();

        // Mint NFT for creator
        mint_nft_for_creator(&mut router, &creator, &nft_contract_addr);

        // An ask is made by the creator, but fails because NFT is not authorized
        let ask = Ask {
            amount: coin(100, NATIVE_TOKEN_DENOM),
        };
        let set_ask = ExecuteMsg::SetAsk {
            collection: nft_contract_addr.clone(),
            token_id: TOKEN_ID.to_string(),
            ask,
        };
        let res =
            router.execute_contract(creator.clone(), nft_marketplace_addr.clone(), &set_ask, &[]);
        assert!(res.is_err());

        // Creator Authorizes NFT
        let approve_msg = Cw721ExecuteMsg::<Empty>::Approve {
            spender: nft_marketplace_addr.to_string(),
            token_id: TOKEN_ID.to_string(),
            expires: None,
        };
        let res = router.execute_contract(
            creator.clone(),
            nft_contract_addr.clone(),
            &approve_msg,
            &[],
        );
        assert!(res.is_ok());

        // Now set_ask succeeds
        let res =
            router.execute_contract(creator.clone(), nft_marketplace_addr.clone(), &set_ask, &[]);
        assert!(res.is_ok());

        // Bidder makes bid with a random token in the same amount as the ask
        router
            .sudo(SudoMsg::Bank({
                BankSudo::Mint {
                    to_address: bidder.to_string(),
                    amount: coins(1000, "random"),
                }
            }))
            .map_err(|err| println!("{:?}", err))
            .ok();
        let bid = Bid {
            amount: coin(100, "random"),
            bidder: bidder.clone(),
            recipient: creator.clone(),
        };
        let set_bid_msg = ExecuteMsg::SetBid {
            collection: nft_contract_addr.clone(),
            token_id: TOKEN_ID.to_string(),
            bid,
        };
        let res = router
            .execute_contract(
                bidder.clone(),
                nft_marketplace_addr.clone(),
                &set_bid_msg,
                &coins(100, "random"),
            )
            .unwrap();

        // A new bid is set, bid is not auto accepted
        assert_eq!("set_bid", res.events[1].attributes[1].value);

        // Bidder makes bid that meets the ask criteria
        let bid = Bid {
            amount: coin(100, NATIVE_TOKEN_DENOM),
            bidder: bidder.clone(),
            recipient: creator.clone(),
        };
        let set_bid_msg = ExecuteMsg::SetBid {
            collection: nft_contract_addr.clone(),
            token_id: TOKEN_ID.to_string(),
            bid,
        };
        let res = router
            .execute_contract(
                bidder.clone(),
                nft_marketplace_addr,
                &set_bid_msg,
                &coins(100, NATIVE_TOKEN_DENOM),
            )
            .unwrap();

        // Bid is accepted, sale has been finalized
        assert_eq!("sale_finalized", res.events[1].attributes[1].value);

        // Check money is transfered
        let creator_native_balances = router.wrap().query_all_balances(creator).unwrap();
        assert_eq!(
            creator_native_balances,
            coins(INITIAL_BALANCE + 100, NATIVE_TOKEN_DENOM)
        );
        let bidder_native_balances = router.wrap().query_all_balances(bidder.clone()).unwrap();
        assert_eq!(
            bidder_native_balances,
            vec![
                coin(1000, "random"),
                coin(INITIAL_BALANCE - 100, NATIVE_TOKEN_DENOM),
            ]
        );

        // Check NFT is transferred
        let query_owner_msg = Cw721QueryMsg::OwnerOf {
            token_id: TOKEN_ID.to_string(),
            include_expired: None,
        };
        let res: OwnerOfResponse = router
            .wrap()
            .query_wasm_smart(nft_contract_addr, &query_owner_msg)
            .unwrap();
        assert_eq!(res.owner, bidder.to_string());
    }

    #[test]
    fn remove_bid_refund() {
        let mut router = mock_app();

        // Setup intial accounts
        let (_owner, bidder, creator) = setup_accounts(&mut router).unwrap();

        // Instantiate and configure contracts
        let (nft_marketplace_addr, nft_contract_addr) =
            setup_contracts(&mut router, &creator).unwrap();

        // Mint NFT for creator
        mint_nft_for_creator(&mut router, &creator, &nft_contract_addr);

        // Bidder makes bid
        let bid = Bid {
            amount: coin(100, NATIVE_TOKEN_DENOM),
            bidder: bidder.clone(),
            recipient: creator,
        };
        let set_bid_msg = ExecuteMsg::SetBid {
            collection: nft_contract_addr.clone(),
            token_id: TOKEN_ID.to_string(),
            bid,
        };

        let res = router.execute_contract(
            bidder.clone(),
            nft_marketplace_addr.clone(),
            &set_bid_msg,
            &coins(100, NATIVE_TOKEN_DENOM),
        );
        assert!(res.is_ok());

        // Bidder sent bid money
        let bidder_native_balances = router.wrap().query_all_balances(bidder.clone()).unwrap();
        assert_eq!(
            bidder_native_balances,
            coins(INITIAL_BALANCE - 100, NATIVE_TOKEN_DENOM)
        );

        // Contract has been paid
        let contract_balances = router
            .wrap()
            .query_all_balances(nft_marketplace_addr.clone())
            .unwrap();
        assert_eq!(contract_balances, coins(100, NATIVE_TOKEN_DENOM));

        // Bidder removes bid
        let remove_bid_msg = ExecuteMsg::RemoveBid {
            collection: nft_contract_addr,
            token_id: TOKEN_ID.to_string(),
            bidder: bidder.clone(),
        };
        let res =
            router.execute_contract(bidder.clone(), nft_marketplace_addr, &remove_bid_msg, &[]);
        assert!(res.is_ok());

        // Bidder has money back
        let bidder_native_balances = router.wrap().query_all_balances(bidder).unwrap();
        assert_eq!(
            bidder_native_balances,
            coins(INITIAL_BALANCE, NATIVE_TOKEN_DENOM)
        );
    }

    #[test]
    fn new_bid_refund() {
        let mut router = mock_app();

        // Setup intial accounts
        let (_owner, bidder, creator) = setup_accounts(&mut router).unwrap();

        // Instantiate and configure contracts
        let (nft_marketplace_addr, nft_contract_addr) =
            setup_contracts(&mut router, &creator).unwrap();

        // Mint NFT for creator
        mint_nft_for_creator(&mut router, &creator, &nft_contract_addr);

        // Bidder makes bid
        let bid = Bid {
            amount: coin(100, NATIVE_TOKEN_DENOM),
            bidder: bidder.clone(),
            recipient: creator.clone(),
        };
        let set_bid_msg = ExecuteMsg::SetBid {
            collection: nft_contract_addr.clone(),
            token_id: TOKEN_ID.to_string(),
            bid,
        };
        let res = router.execute_contract(
            bidder.clone(),
            nft_marketplace_addr.clone(),
            &set_bid_msg,
            &coins(100, NATIVE_TOKEN_DENOM),
        );
        assert!(res.is_ok());

        // Bidder sent bid money
        let bidder_native_balances = router.wrap().query_all_balances(bidder.clone()).unwrap();
        assert_eq!(
            bidder_native_balances,
            coins(INITIAL_BALANCE - 100, NATIVE_TOKEN_DENOM)
        );

        // Contract has been paid
        let contract_balances = router
            .wrap()
            .query_all_balances(nft_marketplace_addr.clone())
            .unwrap();
        assert_eq!(contract_balances, coins(100, NATIVE_TOKEN_DENOM));

        // Bidder makes higher bid
        let bid = Bid {
            amount: coin(150, NATIVE_TOKEN_DENOM),
            bidder: bidder.clone(),
            recipient: creator,
        };
        let set_bid_msg = ExecuteMsg::SetBid {
            collection: nft_contract_addr.clone(),
            token_id: TOKEN_ID.to_string(),
            bid: bid.clone(),
        };
        let res = router.execute_contract(
            bidder.clone(),
            nft_marketplace_addr.clone(),
            &set_bid_msg,
            &coins(150, NATIVE_TOKEN_DENOM),
        );
        assert!(res.is_ok());

        // Bidder has money back
        let bidder_native_balances = router.wrap().query_all_balances(bidder.clone()).unwrap();
        assert_eq!(
            bidder_native_balances,
            coins(INITIAL_BALANCE - 150, NATIVE_TOKEN_DENOM)
        );

        // Contract has been paid
        let contract_balances = router
            .wrap()
            .query_all_balances(nft_marketplace_addr.clone())
            .unwrap();
        assert_eq!(contract_balances, coins(150, NATIVE_TOKEN_DENOM));

        // Check new bid has been saved
        let query_bid_msg = QueryMsg::Bid {
            collection: nft_contract_addr,
            token_id: TOKEN_ID.to_string(),
            bidder,
        };
        let res: BidResponse = router
            .wrap()
            .query_wasm_smart(nft_marketplace_addr, &query_bid_msg)
            .unwrap();
        assert_eq!(Some(bid), res.bid);
    }

    #[test]
    fn royalties() {
        let mut router = mock_app();

        // Setup intial accounts
        let (curator, bidder, creator) = setup_accounts(&mut router).unwrap();

        // Instantiate and configure contracts
        let (nft_marketplace_addr, _) = setup_contracts(&mut router, &creator).unwrap();

        // Setup media contract with 10% royalties to a curator
        let sg721_id = router.store_code(contract_sg721());
        let msg = sg721::msg::InstantiateMsg {
            name: String::from("Test Coin"),
            symbol: String::from("TEST"),
            minter: creator.to_string(),
            config: Some(Config {
                contract_uri: Some(String::from("https://bafyreibvxty5gjyeedk7or7tahyrzgbrwjkolpairjap3bmegvcjdipt74.ipfs.dweb.link/metadata.json")),
                creator: creator.clone(),
                royalties: Some(RoyaltyInfo {
                    payment_address: curator.clone(),
                    share: Decimal::percent(10),
                }),
            }),
        };
        let nft_contract_addr = router
            .instantiate_contract(sg721_id, creator.clone(), &msg, &[], "NFT", None)
            .unwrap();

        // Mint NFT for creator
        let mint_for_creator_msg = Cw721ExecuteMsg::Mint(MintMsg {
            token_id: TOKEN_ID.to_string(),
            owner: creator.to_string(),
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

        // Creator Authorizes NFT
        let approve_msg = Cw721ExecuteMsg::<Empty>::Approve {
            spender: nft_marketplace_addr.to_string(),
            token_id: TOKEN_ID.to_string(),
            expires: None,
        };
        let res = router.execute_contract(
            creator.clone(),
            nft_contract_addr.clone(),
            &approve_msg,
            &[],
        );
        assert!(res.is_ok());

        // An ask is made by the creator
        let ask = Ask {
            amount: coin(100, NATIVE_TOKEN_DENOM),
        };
        let set_ask = ExecuteMsg::SetAsk {
            collection: nft_contract_addr.clone(),
            token_id: TOKEN_ID.to_string(),
            ask,
        };
        let res =
            router.execute_contract(creator.clone(), nft_marketplace_addr.clone(), &set_ask, &[]);
        assert!(res.is_ok());

        // Bidder makes bid
        let bid = Bid {
            amount: coin(100, NATIVE_TOKEN_DENOM),
            bidder: bidder.clone(),
            recipient: creator.clone(),
        };
        let set_bid_msg = ExecuteMsg::SetBid {
            collection: nft_contract_addr.clone(),
            token_id: TOKEN_ID.to_string(),
            bid,
        };
        let res = router.execute_contract(
            bidder.clone(),
            nft_marketplace_addr,
            &set_bid_msg,
            &coins(100, NATIVE_TOKEN_DENOM),
        );
        assert!(res.is_ok());

        // Check money is transfered correctly and royalties paid
        let curator_native_balances = router.wrap().query_all_balances(curator).unwrap();
        assert_eq!(
            curator_native_balances,
            coins(INITIAL_BALANCE + 10, NATIVE_TOKEN_DENOM)
        );
        let creator_native_balances = router.wrap().query_all_balances(creator).unwrap();
        assert_eq!(
            creator_native_balances,
            coins(INITIAL_BALANCE + 90, NATIVE_TOKEN_DENOM)
        );
        let bidder_native_balances = router.wrap().query_all_balances(bidder.clone()).unwrap();
        assert_eq!(
            bidder_native_balances,
            coins(INITIAL_BALANCE - 100, NATIVE_TOKEN_DENOM)
        );

        // Check NFT is transferred
        let query_owner_msg = Cw721QueryMsg::OwnerOf {
            token_id: TOKEN_ID.to_string(),
            include_expired: None,
        };
        let res: OwnerOfResponse = router
            .wrap()
            .query_wasm_smart(nft_contract_addr, &query_owner_msg)
            .unwrap();
        assert_eq!(res.owner, bidder.to_string());
    }
}
