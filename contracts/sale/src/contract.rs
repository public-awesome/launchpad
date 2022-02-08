#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    has_coins, to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Empty,
    Env, MessageInfo, Order, Reply, ReplyOn, Response, StdError, StdResult, SubMsg, WasmMsg,
};
use cw2::set_contract_version;
use cw721::TokensResponse as Cw721TokensResponse;
use cw721_base::{msg::ExecuteMsg as Cw721ExecuteMsg, MintMsg};
use cw_utils::{parse_reply_instantiate_data, Expiration};
use sg721::msg::{InstantiateMsg as Sg721InstantiateMsg, QueryMsg as Sg721QueryMsg};
use url::Url;

use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, StartTimeResponse, UpdateWhitelistMsg,
    WhitelistAddressesResponse, WhitelistExpirationResponse,
};
use crate::state::{
    Config, CONFIG, NUM_WHITELIST_ADDRS, SG721_ADDRESS, TOKEN_ID_INDEX, WHITELIST_ADDRS,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sale";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const INSTANTIATE_SG721_REPLY_ID: u64 = 1;
const MAX_WHITELIST_ADDRS_LENGTH: u32 = 15000;
const MAX_PER_ADDRESS_LIMIT: u64 = 30;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    if let Some(per_address_limit) = msg.per_address_limit {
        // Check per address limit is valid
        if per_address_limit > MAX_PER_ADDRESS_LIMIT {
            return Err(ContractError::InvalidPerAddressLimit {});
        }
    }

    // Check that base_token_uri is a valid IPFS uri
    let parsed_token_uri = Url::parse(&msg.base_token_uri)?;
    if parsed_token_uri.scheme() != "ipfs" {
        return Err(ContractError::InvalidBaseTokenURI {});
    }

    let config = Config {
        admin: info.sender,
        base_token_uri: msg.base_token_uri,
        num_tokens: msg.num_tokens,
        sg721_code_id: msg.sg721_code_id,
        unit_price: msg.unit_price,
        whitelist_expiration: msg.whitelist_expiration,
        start_time: msg.start_time,
        per_address_limit: msg.per_address_limit,
    };
    CONFIG.save(deps.storage, &config)?;

    // Set whitelist addresses and num_whitelist_addresses
    if let Some(whitelist_addresses) = msg.whitelist_addresses {
        // Check length of whitelist addresses is not greater than max allowed
        if MAX_WHITELIST_ADDRS_LENGTH <= (whitelist_addresses.len() as u32) {
            return Err(ContractError::MaxWhitelistAddressLengthExceeded {});
        }

        for whitelist_address in whitelist_addresses.clone().into_iter() {
            WHITELIST_ADDRS.save(deps.storage, whitelist_address, &Empty {})?;
        }
        NUM_WHITELIST_ADDRS.save(deps.storage, &(whitelist_addresses.len() as u32))?;
    }

    // Set Token ID index
    TOKEN_ID_INDEX.save(deps.storage, &0)?;

    let sub_msgs: Vec<SubMsg> = vec![SubMsg {
        msg: WasmMsg::Instantiate {
            code_id: msg.sg721_code_id,
            msg: to_binary(&Sg721InstantiateMsg {
                name: msg.sg721_instantiate_msg.name,
                symbol: msg.sg721_instantiate_msg.symbol,
                minter: env.contract.address.to_string(),
                collection_info: msg.sg721_instantiate_msg.collection_info,
            })?,
            funds: vec![],
            admin: None,
            label: String::from("Instantiate fixed price NFT contract"),
        }
        .into(),
        id: INSTANTIATE_SG721_REPLY_ID,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    }];

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_submessages(sub_msgs))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Mint {} => execute_mint(deps, env, info),
        ExecuteMsg::UpdateWhitelist(update_whitelist_msg) => {
            execute_update_whitelist(deps, env, info, update_whitelist_msg)
        }
        ExecuteMsg::UpdateWhitelistExpiration(expiration) => {
            execute_update_whitelist_expiration(deps, env, info, expiration)
        }
        ExecuteMsg::UpdateStartTime(expiration) => {
            execute_update_start_time(deps, env, info, expiration)
        }
        ExecuteMsg::UpdatePerAddressLimit { per_address_limit } => {
            execute_update_per_address_limit(deps, env, info, per_address_limit)
        }
        ExecuteMsg::MintFor { recipient } => execute_mint_for(deps, env, info, recipient),
    }
}

pub fn execute_mint(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let sg721_address = SG721_ADDRESS.load(deps.storage)?;
    let mut token_id_index = TOKEN_ID_INDEX.load(deps.storage)?;
    let allowlist = WHITELIST_ADDRS.has(deps.storage, info.sender.to_string());
    if let Some(whitelist_expiration) = config.whitelist_expiration {
        // Check if whitelist not expired and sender is not whitelisted
        if !whitelist_expiration.is_expired(&env.block) && !allowlist {
            return Err(ContractError::NotWhitelisted {});
        }
    }

    // Check funds sent is correct amount
    if !has_coins(&info.funds, &config.unit_price) {
        return Err(ContractError::NotEnoughFunds {});
    }

    // Check if over max tokens
    if token_id_index >= config.num_tokens {
        return Err(ContractError::SoldOut {});
    }

    if let Some(start_time) = config.start_time {
        // Check if after start_time
        if !start_time.is_expired(&env.block) {
            return Err(ContractError::BeforeMintStartTime {});
        }
    }

    // Check if already minted max per address limit
    if let Some(per_address_limit) = config.per_address_limit {
        let tokens: Cw721TokensResponse = deps.querier.query_wasm_smart(
            sg721_address.to_string(),
            &Sg721QueryMsg::Tokens {
                owner: info.sender.to_string(),
                start_after: None,
                limit: Some(MAX_PER_ADDRESS_LIMIT as u32),
            },
        )?;
        if tokens.tokens.len() >= per_address_limit as usize {
            return Err(ContractError::MaxPerAddressLimitExceeded {});
        }
    }

    let mut msgs: Vec<CosmosMsg> = vec![];

    let mint_msg = Cw721ExecuteMsg::Mint(MintMsg::<Empty> {
        token_id: token_id_index.to_string(),
        owner: info.sender.to_string(),
        token_uri: Some(format!("{}/{}", config.base_token_uri, token_id_index)),
        extension: Empty {},
    });

    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: sg721_address.to_string(),
        msg: to_binary(&mint_msg)?,
        funds: vec![],
    });
    msgs.append(&mut vec![msg]);

    // Increase token ID index by one
    token_id_index += 1;
    TOKEN_ID_INDEX.save(deps.storage, &token_id_index)?;

    // Check if token supports Royalties
    let royalty: Result<sg721::msg::RoyaltyResponse, StdError> = deps
        .querier
        .query_wasm_smart(sg721_address, &Sg721QueryMsg::Royalties {});

    // Add payout messages
    match royalty {
        Ok(royalty) => {
            // If token supports royalities, payout shares
            if let Some(royalty) = royalty.royalty {
                // Can't assume index 0 of index.funds is the correct coin
                let funds = info.funds.iter().find(|x| *x == &config.unit_price);
                if let Some(funds) = funds {
                    // Calculate royalty share and create Bank msg
                    let royalty_share_msg = BankMsg::Send {
                        to_address: royalty.payment_address.to_string(),
                        amount: vec![Coin {
                            amount: funds.amount * royalty.share,
                            denom: funds.denom.clone(),
                        }],
                    };
                    msgs.append(&mut vec![royalty_share_msg.into()]);

                    // Calculate seller share and create Bank msg
                    let seller_share_msg = BankMsg::Send {
                        to_address: config.admin.to_string(),
                        amount: vec![Coin {
                            amount: funds.amount * (Decimal::one() - royalty.share),
                            denom: funds.denom.clone(),
                        }],
                    };
                    msgs.append(&mut vec![seller_share_msg.into()]);
                }
            }
        }
        Err(_) => {
            // If token doesn't support royalties, pay seller in full
            let seller_share_msg = BankMsg::Send {
                to_address: config.admin.to_string(),
                amount: info.funds,
            };
            msgs.append(&mut vec![seller_share_msg.into()]);
        }
    }
    Ok(Response::default().add_messages(msgs))
}

pub fn execute_mint_for(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    recipient: Addr,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let sg721_address = SG721_ADDRESS.load(deps.storage)?;
    let mut token_id_index = TOKEN_ID_INDEX.load(deps.storage)?;

    // Check only admin
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    // Check if over max tokens
    if token_id_index >= config.num_tokens {
        return Err(ContractError::SoldOut {});
    }

    let mint_msg = Cw721ExecuteMsg::Mint(MintMsg::<Empty> {
        token_id: token_id_index.to_string(),
        owner: recipient.to_string(),
        token_uri: Some(format!("{}/{}", config.base_token_uri, token_id_index)),
        extension: Empty {},
    });

    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: sg721_address.to_string(),
        msg: to_binary(&mint_msg)?,
        funds: vec![],
    });

    // Increase token ID index by one
    token_id_index += 1;
    TOKEN_ID_INDEX.save(deps.storage, &token_id_index)?;

    Ok(Response::default().add_message(msg))
}

pub fn execute_update_whitelist(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    update_whitelist_msg: UpdateWhitelistMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut num_whitelist_addresses = NUM_WHITELIST_ADDRS.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    // Add whitelist addresses
    if let Some(add_whitelist_addrs) = update_whitelist_msg.add_addresses {
        if MAX_WHITELIST_ADDRS_LENGTH
            <= (add_whitelist_addrs.len() as u32 + num_whitelist_addresses)
        {
            return Err(ContractError::MaxWhitelistAddressLengthExceeded {});
        }
        for whitelist_address in add_whitelist_addrs.clone().into_iter() {
            WHITELIST_ADDRS.save(deps.storage, whitelist_address, &Empty {})?;
        }
        num_whitelist_addresses += add_whitelist_addrs.len() as u32;
    }

    // Remove whitelist addresses
    if let Some(remove_whitelist_addrs) = update_whitelist_msg.remove_addresses {
        for whitelist_address in remove_whitelist_addrs.clone().into_iter() {
            WHITELIST_ADDRS.remove(deps.storage, whitelist_address);
        }
        num_whitelist_addresses -= remove_whitelist_addrs.len() as u32;
    }

    NUM_WHITELIST_ADDRS.save(deps.storage, &num_whitelist_addresses)?;

    Ok(Response::new().add_attribute("method", "updated_whitelist_addresses"))
}

pub fn execute_update_whitelist_expiration(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    whitelist_expiration: Expiration,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    config.whitelist_expiration = Some(whitelist_expiration);
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attribute("method", "updated_whitelist_expiration"))
}

pub fn execute_update_start_time(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    start_time: Expiration,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    config.start_time = Some(start_time);
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attribute("method", "updated_start_time"))
}

pub fn execute_update_per_address_limit(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    per_address_limit: u64,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    if per_address_limit > MAX_PER_ADDRESS_LIMIT {
        return Err(ContractError::InvalidPerAddressLimit {});
    }
    config.per_address_limit = Some(per_address_limit);
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attribute("method", "updated_per_address_limit"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&query_config(deps)?),
        QueryMsg::GetWhitelistAddresses {} => to_binary(&query_whitelist_addresses(deps)?),
        QueryMsg::GetWhitelistExpiration {} => to_binary(&query_whitelist_expiration(deps)?),
        QueryMsg::GetStartTime {} => to_binary(&query_start_time(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    let sg721_address = SG721_ADDRESS.load(deps.storage)?;
    let unused_token_id = TOKEN_ID_INDEX.load(deps.storage)?;

    Ok(ConfigResponse {
        admin: config.admin,
        base_token_uri: config.base_token_uri,
        sg721_address,
        sg721_code_id: config.sg721_code_id,
        num_tokens: config.num_tokens,
        unit_price: config.unit_price,
        unused_token_id,
        per_address_limit: config.per_address_limit,
    })
}

fn query_whitelist_addresses(deps: Deps) -> StdResult<WhitelistAddressesResponse> {
    let addrs: StdResult<Vec<String>> = WHITELIST_ADDRS
        .keys(deps.storage, None, None, Order::Ascending)
        .take_while(|x| x.is_ok())
        .collect::<StdResult<Vec<String>>>();
    Ok(WhitelistAddressesResponse { addresses: addrs? })
}

fn query_whitelist_expiration(deps: Deps) -> StdResult<WhitelistExpirationResponse> {
    let config = CONFIG.load(deps.storage)?;
    if let Some(expiration) = config.whitelist_expiration {
        Ok(WhitelistExpirationResponse {
            expiration_time: expiration.to_string(),
        })
    } else {
        Err(StdError::GenericErr {
            msg: "whitelist expiration not found".to_string(),
        })
    }
}

fn query_start_time(deps: Deps) -> StdResult<StartTimeResponse> {
    let config = CONFIG.load(deps.storage)?;
    if let Some(expiration) = config.start_time {
        Ok(StartTimeResponse {
            start_time: expiration.to_string(),
        })
    } else {
        Err(StdError::GenericErr {
            msg: "start time not found".to_string(),
        })
    }
}

// Reply callback triggered from cw721 contract instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    if msg.id != INSTANTIATE_SG721_REPLY_ID {
        return Err(ContractError::InvalidReplyID {});
    }

    let reply = parse_reply_instantiate_data(msg);
    match reply {
        Ok(res) => {
            SG721_ADDRESS.save(deps.storage, &Addr::unchecked(res.contract_address))?;
            Ok(Response::default())
        }
        Err(_) => Err(ContractError::InstantiateSg721Error {}),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
    use cosmwasm_std::{coin, coins, Decimal, Timestamp};
    use cw721::{Cw721QueryMsg, OwnerOfResponse};
    use cw_multi_test::{App, BankSudo, Contract, ContractWrapper, Executor, SudoMsg};
    use sg721::state::{CollectionInfo, RoyaltyInfo};

    const DENOM: &str = "ustars";
    const INITIAL_BALANCE: u128 = 2000;
    const PRICE: u128 = 10;

    fn mock_app() -> App {
        App::default()
    }

    pub fn contract_sale() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        )
        .with_reply(crate::contract::reply);
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

    // Upload contract code and instantiate sale contract
    fn setup_sale_contract(
        router: &mut App,
        creator: &Addr,
        num_tokens: u64,
    ) -> Result<(Addr, ConfigResponse), ContractError> {
        // Upload contract code
        let sg721_code_id = router.store_code(contract_sg721());
        let sale_code_id = router.store_code(contract_sale());

        // Instantiate sale contract
        let msg = InstantiateMsg {
            unit_price: coin(PRICE, DENOM),
            num_tokens,
            whitelist_expiration: None,
            whitelist_addresses: Some(vec![String::from("VIPcollector")]),
            start_time: None,
            per_address_limit: None,
            base_token_uri: "ipfs://QmYxw1rURvnbQbBRTfmVaZtxSrkrfsbodNzibgBrVrUrtN".to_string(),
            sg721_code_id,
            sg721_instantiate_msg: Sg721InstantiateMsg {
                name: String::from("TEST"),
                symbol: String::from("TEST"),
                minter: creator.to_string(),
                collection_info: CollectionInfo {
                    contract_uri: String::from("test"),
                    creator: creator.clone(),
                    royalties: Some(RoyaltyInfo {
                        payment_address: creator.clone(),
                        share: Decimal::percent(10),
                    }),
                },
            },
        };
        let sale_addr = router
            .instantiate_contract(sale_code_id, creator.clone(), &msg, &[], "Sale", None)
            .unwrap();

        let config: ConfigResponse = router
            .wrap()
            .query_wasm_smart(sale_addr.clone(), &QueryMsg::GetConfig {})
            .unwrap();

        Ok((sale_addr, config))
    }

    // Add a creator account with initial balances
    fn setup_accounts(router: &mut App) -> Result<(Addr, Addr), ContractError> {
        let buyer: Addr = Addr::unchecked("buyer");
        let creator: Addr = Addr::unchecked("creator");
        let funds: Vec<Coin> = coins(INITIAL_BALANCE, DENOM);
        router
            .sudo(SudoMsg::Bank({
                BankSudo::Mint {
                    to_address: creator.to_string(),
                    amount: funds.clone(),
                }
            }))
            .map_err(|err| println!("{:?}", err))
            .ok();

        router
            .sudo(SudoMsg::Bank({
                BankSudo::Mint {
                    to_address: buyer.to_string(),
                    amount: funds.clone(),
                }
            }))
            .map_err(|err| println!("{:?}", err))
            .ok();

        // Check native balances
        let creator_native_balances = router.wrap().query_all_balances(creator.clone()).unwrap();
        assert_eq!(creator_native_balances, funds);

        // Check native balances
        let buyer_native_balances = router.wrap().query_all_balances(buyer.clone()).unwrap();
        assert_eq!(buyer_native_balances, funds);

        Ok((creator, buyer))
    }

    #[test]
    fn initialization() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        // Invalid uri returns error
        let info = mock_info("creator", &coins(INITIAL_BALANCE, DENOM));
        let msg = InstantiateMsg {
            unit_price: coin(PRICE, DENOM),
            num_tokens: 100,
            whitelist_expiration: None,
            whitelist_addresses: Some(vec![String::from("VIPcollector")]),
            start_time: None,
            per_address_limit: None,
            base_token_uri: "https://QmYxw1rURvnbQbBRTfmVaZtxSrkrfsbodNzibgBrVrUrtN".to_string(),
            sg721_code_id: 1,
            sg721_instantiate_msg: Sg721InstantiateMsg {
                name: String::from("TEST"),
                symbol: String::from("TEST"),
                minter: info.sender.to_string(),
                collection_info: CollectionInfo {
                    contract_uri: String::from("test"),
                    creator: info.sender.clone(),
                    royalties: Some(RoyaltyInfo {
                        payment_address: info.sender.clone(),
                        share: Decimal::percent(10),
                    }),
                },
            },
        };
        let res = instantiate(deps.as_mut(), mock_env(), info, msg);
        assert!(res.is_err());
    }

    #[test]
    fn happy_path() {
        let mut router = mock_app();
        let (creator, buyer) = setup_accounts(&mut router).unwrap();
        let num_tokens: u64 = 2;
        let (sale_addr, config) = setup_sale_contract(&mut router, &creator, num_tokens).unwrap();

        // Succeeds if funds are sent
        let mint_msg = ExecuteMsg::Mint {};
        let res = router.execute_contract(
            buyer.clone(),
            sale_addr.clone(),
            &mint_msg,
            &coins(PRICE, DENOM),
        );
        assert!(res.is_ok());

        // Balances are correct
        let creator_native_balances = router.wrap().query_all_balances(creator.clone()).unwrap();
        assert_eq!(
            creator_native_balances,
            coins(INITIAL_BALANCE + PRICE, DENOM)
        );
        let buyer_native_balances = router.wrap().query_all_balances(buyer.clone()).unwrap();
        assert_eq!(buyer_native_balances, coins(INITIAL_BALANCE - PRICE, DENOM));

        // Check NFT is transferred
        let query_owner_msg = Cw721QueryMsg::OwnerOf {
            token_id: String::from("0"),
            include_expired: None,
        };
        let res: OwnerOfResponse = router
            .wrap()
            .query_wasm_smart(config.sg721_address.clone(), &query_owner_msg)
            .unwrap();
        assert_eq!(res.owner, buyer.to_string());

        // Buyer can't call MintFor
        let mint_for_msg = ExecuteMsg::MintFor {
            recipient: buyer.clone(),
        };
        let res = router.execute_contract(
            buyer.clone(),
            sale_addr.clone(),
            &mint_for_msg,
            &coins(PRICE, DENOM),
        );
        assert!(res.is_err());

        // Creator mints an extra NFT for the buyer (who is a friend)
        let res = router.execute_contract(
            creator.clone(),
            sale_addr.clone(),
            &mint_for_msg,
            &coins(PRICE, DENOM),
        );
        assert!(res.is_ok());

        // Check that NFT is transferred
        let query_owner_msg = Cw721QueryMsg::OwnerOf {
            token_id: String::from("1"),
            include_expired: None,
        };
        let res: OwnerOfResponse = router
            .wrap()
            .query_wasm_smart(config.sg721_address, &query_owner_msg)
            .unwrap();
        assert_eq!(res.owner, buyer.to_string());

        // Errors if sold out
        let mint_msg = ExecuteMsg::Mint {};
        let res =
            router.execute_contract(buyer, sale_addr.clone(), &mint_msg, &coins(PRICE, DENOM));
        assert!(res.is_err());

        // Creator can't use MintFor if sold out
        let res = router.execute_contract(creator, sale_addr, &mint_for_msg, &coins(PRICE, DENOM));
        assert!(res.is_err());
    }

    #[test]
    fn whitelist_access_len_add_remove_expiration() {
        let mut router = mock_app();
        let (creator, buyer) = setup_accounts(&mut router).unwrap();
        let num_tokens: u64 = 1;
        let (sale_addr, _config) = setup_sale_contract(&mut router, &creator, num_tokens).unwrap();
        const EXPIRATION_TIME: Timestamp = Timestamp::from_seconds(100000 + 10);

        // set block info
        let mut block = router.block_info();
        block.time = Timestamp::from_seconds(100000);
        router.set_block(block);

        // update whitelist_expiration fails if not admin
        let whitelist_msg = ExecuteMsg::UpdateWhitelistExpiration(Expiration::Never {});
        let res = router.execute_contract(
            buyer.clone(),
            sale_addr.clone(),
            &whitelist_msg,
            &coins(PRICE, DENOM),
        );
        assert!(res.is_err());

        // enable whitelist
        let whitelist_msg =
            ExecuteMsg::UpdateWhitelistExpiration(Expiration::AtTime(EXPIRATION_TIME));
        let res = router.execute_contract(
            creator.clone(),
            sale_addr.clone(),
            &whitelist_msg,
            &coins(PRICE, DENOM),
        );
        assert!(res.is_ok());

        // mint fails, buyer is not on whitelist
        let mint_msg = ExecuteMsg::Mint {};
        let res = router.execute_contract(
            buyer.clone(),
            sale_addr.clone(),
            &mint_msg,
            &coins(PRICE, DENOM),
        );
        assert!(res.is_err());

        // fails, add too many whitelist addresses
        let over_max_limit_whitelist_addrs =
            vec!["addr".to_string(); MAX_WHITELIST_ADDRS_LENGTH as usize + 10];
        let whitelist: Option<Vec<String>> = Some(over_max_limit_whitelist_addrs);
        let add_whitelist_msg = UpdateWhitelistMsg {
            add_addresses: whitelist,
            remove_addresses: None,
        };
        let update_whitelist_msg = ExecuteMsg::UpdateWhitelist(add_whitelist_msg);
        let res = router.execute_contract(
            creator.clone(),
            sale_addr.clone(),
            &update_whitelist_msg,
            &coins(PRICE, DENOM),
        );
        assert!(res.is_err());

        // add buyer to whitelist
        let whitelist: Option<Vec<String>> = Some(vec![buyer.clone().into_string()]);
        let add_whitelist_msg = UpdateWhitelistMsg {
            add_addresses: whitelist,
            remove_addresses: None,
        };
        let update_whitelist_msg = ExecuteMsg::UpdateWhitelist(add_whitelist_msg);
        let res = router.execute_contract(
            creator.clone(),
            sale_addr.clone(),
            &update_whitelist_msg,
            &coins(PRICE, DENOM),
        );
        assert!(res.is_ok());

        // query whitelist, confirm buyer on allowlist
        let allowlist: WhitelistAddressesResponse = router
            .wrap()
            .query_wasm_smart(sale_addr.clone(), &QueryMsg::GetWhitelistAddresses {})
            .unwrap();
        assert!(allowlist.addresses.contains(&"buyer".to_string()));

        // query whitelist_expiration, confirm not expired
        let expiration: WhitelistExpirationResponse = router
            .wrap()
            .query_wasm_smart(sale_addr.clone(), &QueryMsg::GetWhitelistExpiration {})
            .unwrap();
        assert_eq!(
            "expiration time: ".to_owned() + &EXPIRATION_TIME.to_string(),
            expiration.expiration_time
        );

        // mint succeeds
        let mint_msg = ExecuteMsg::Mint {};
        let res = router.execute_contract(
            buyer.clone(),
            sale_addr.clone(),
            &mint_msg,
            &coins(PRICE, DENOM),
        );
        assert!(res.is_ok());

        // remove buyer from whitelist
        let remove_whitelist: Option<Vec<String>> = Some(vec![buyer.clone().into_string()]);
        let remove_whitelist_msg = UpdateWhitelistMsg {
            add_addresses: None,
            remove_addresses: remove_whitelist,
        };
        let update_whitelist_msg = ExecuteMsg::UpdateWhitelist(remove_whitelist_msg);
        let res = router.execute_contract(
            creator.clone(),
            sale_addr.clone(),
            &update_whitelist_msg,
            &coins(PRICE, DENOM),
        );
        assert!(res.is_ok());

        // mint fails
        let mint_msg = ExecuteMsg::Mint {};
        let res = router.execute_contract(buyer, sale_addr, &mint_msg, &coins(PRICE, DENOM));
        assert!(res.is_err());
    }

    #[test]
    fn before_start_time() {
        let mut router = mock_app();
        let (creator, buyer) = setup_accounts(&mut router).unwrap();
        let num_tokens: u64 = 1;
        let (sale_addr, _config) = setup_sale_contract(&mut router, &creator, num_tokens).unwrap();
        const START_TIME: Timestamp = Timestamp::from_seconds(100000 + 10);

        // set block info
        let mut block = router.block_info();
        block.time = Timestamp::from_seconds(100000);
        router.set_block(block);

        // set start_time fails if not admin
        let start_time_msg = ExecuteMsg::UpdateStartTime(Expiration::Never {});
        let res = router.execute_contract(
            buyer.clone(),
            sale_addr.clone(),
            &start_time_msg,
            &coins(PRICE, DENOM),
        );
        assert!(res.is_err());

        // if block before start_time, throw error
        let start_time_msg = ExecuteMsg::UpdateStartTime(Expiration::AtTime(START_TIME));
        let res = router.execute_contract(
            creator.clone(),
            sale_addr.clone(),
            &start_time_msg,
            &coins(PRICE, DENOM),
        );
        assert!(res.is_ok());

        let mint_msg = ExecuteMsg::Mint {};
        let res = router.execute_contract(
            buyer.clone(),
            sale_addr.clone(),
            &mint_msg,
            &coins(PRICE, DENOM),
        );
        assert!(res.is_err());

        // query start_time, confirm expired
        let start_time_response: StartTimeResponse = router
            .wrap()
            .query_wasm_smart(sale_addr.clone(), &QueryMsg::GetStartTime {})
            .unwrap();
        assert_eq!(
            "expiration time: ".to_owned() + &START_TIME.to_string(),
            start_time_response.start_time
        );

        // set block forward, after start time. mint succeeds
        let mut block = router.block_info();
        block.time = START_TIME.plus_seconds(10);
        router.set_block(block);

        // mint succeeds
        let mint_msg = ExecuteMsg::Mint {};
        let res = router.execute_contract(buyer, sale_addr, &mint_msg, &coins(PRICE, DENOM));
        assert!(res.is_ok());
    }

    #[test]
    fn per_address_limit() {
        let mut router = mock_app();
        let (creator, buyer) = setup_accounts(&mut router).unwrap();
        let num_tokens = 2;
        let (sale_addr, _config) = setup_sale_contract(&mut router, &creator, num_tokens).unwrap();

        // set limit, check unauthorized
        let per_address_limit_msg = ExecuteMsg::UpdatePerAddressLimit {
            per_address_limit: 30,
        };
        let res = router.execute_contract(
            buyer.clone(),
            sale_addr.clone(),
            &per_address_limit_msg,
            &coins(PRICE, DENOM),
        );
        assert!(res.is_err());

        // set limit, invalid limit over max
        let per_address_limit_msg = ExecuteMsg::UpdatePerAddressLimit {
            per_address_limit: 100,
        };
        let res = router.execute_contract(
            creator.clone(),
            sale_addr.clone(),
            &per_address_limit_msg,
            &coins(PRICE, DENOM),
        );
        assert!(res.is_err());

        // set limit, mint fails, over max
        let per_address_limit_msg = ExecuteMsg::UpdatePerAddressLimit {
            per_address_limit: 1,
        };
        let res = router.execute_contract(
            creator.clone(),
            sale_addr.clone(),
            &per_address_limit_msg,
            &coins(PRICE, DENOM),
        );
        assert!(res.is_ok());

        // first mint succeeds
        let mint_msg = ExecuteMsg::Mint {};
        let res = router.execute_contract(
            buyer.clone(),
            sale_addr.clone(),
            &mint_msg,
            &coins(PRICE, DENOM),
        );
        assert!(res.is_ok());

        // second mint fails from exceeding per address limit
        let mint_msg = ExecuteMsg::Mint {};
        let res = router.execute_contract(buyer, sale_addr, &mint_msg, &coins(PRICE, DENOM));
        assert!(res.is_err());
    }

    #[test]
    fn unhappy_path() {
        let mut router = mock_app();
        let (creator, buyer) = setup_accounts(&mut router).unwrap();
        let num_tokens: u64 = 1;
        let (sale_addr, _config) = setup_sale_contract(&mut router, &creator, num_tokens).unwrap();

        // Fails if too little funds are sent
        let mint_msg = ExecuteMsg::Mint {};
        let res = router.execute_contract(
            buyer.clone(),
            sale_addr.clone(),
            &mint_msg,
            &coins(1, DENOM),
        );
        assert!(res.is_err());

        // Fails if too many funds are sent
        let mint_msg = ExecuteMsg::Mint {};
        let res = router.execute_contract(
            buyer.clone(),
            sale_addr.clone(),
            &mint_msg,
            &coins(11111, DENOM),
        );
        assert!(res.is_err());

        // Fails wrong denom is sent
        let mint_msg = ExecuteMsg::Mint {};
        let res = router.execute_contract(buyer, sale_addr, &mint_msg, &coins(PRICE, "uatom"));
        assert!(res.is_err());
    }
}
